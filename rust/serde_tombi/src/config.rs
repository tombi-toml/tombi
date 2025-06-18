use tombi_ast::AstNode;
use tombi_config::{
    Config, TomlVersion, PYPROJECT_FILENAME, TOMBI_CONFIG_FILENAME, TOMBI_CONFIG_TOML_VERSION,
    TOMBI_USER_CONFIG_FILENAME,
};
use tombi_url::url_to_file_path;

/// Parse the TOML text into a `Config` struct.
///
/// When executing [crate::from_str_async], it is necessary to obtain the Config to determine the TOML version.
/// If [crate::from_str_async] is used to parse the Config, it will cause a stack overflow due to circular references.
/// Therefore, [crate::config::from_str], which does not use schema_store and is not async, is called to prevent stack overflow.
///
/// This function is not public and is only used internally.
pub(crate) fn from_str(
    toml_text: &str,
    config_path: &std::path::Path,
) -> Result<Config, crate::de::Error> {
    let deserializer = crate::Deserializer::builder()
        .config_path(config_path)
        .build();

    let toml_version = TOMBI_CONFIG_TOML_VERSION;
    let parsed = tombi_parser::parse(toml_text, toml_version);
    let root = tombi_ast::Root::cast(parsed.syntax_node()).expect("AST Root must be present");
    // Check if there are any parsing errors
    if !parsed.errors.is_empty() {
        return Err(parsed.errors.into());
    }

    deserializer.from_document(deserializer.try_to_document(root, TOMBI_CONFIG_TOML_VERSION)?)
}

#[doc(hidden)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
struct PyProjectToml {
    tool: Option<Tool>,
}

impl PyProjectToml {
    fn from_str(toml_text: &str, config_path: &std::path::Path) -> Result<Self, crate::de::Error> {
        let deserializer = crate::Deserializer::builder()
            .config_path(config_path)
            .build();

        let toml_version = TOMBI_CONFIG_TOML_VERSION;
        let parsed = tombi_parser::parse(toml_text, toml_version);
        let root = tombi_ast::Root::cast(parsed.syntax_node()).expect("AST Root must be present");
        // Check if there are any parsing errors
        if !parsed.errors.is_empty() {
            return Err(parsed.errors.into());
        }

        deserializer.from_document(deserializer.try_to_document(root, TomlVersion::V1_0_0)?)
    }
}

#[doc(hidden)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
struct Tool {
    tombi: Option<Config>,
}

pub fn try_from_path<P: AsRef<std::path::Path>>(
    config_path: P,
) -> Result<Option<Config>, tombi_config::Error> {
    let config_path = config_path.as_ref();

    if !config_path.exists() {
        return Err(tombi_config::Error::ConfigFileNotFound {
            config_path: config_path.to_owned(),
        });
    }

    let Ok(config_text) = std::fs::read_to_string(config_path) else {
        return Err(tombi_config::Error::ConfigFileReadFailed {
            config_path: config_path.to_owned(),
        });
    };

    match config_path.file_name().and_then(|name| name.to_str()) {
        Some(TOMBI_CONFIG_FILENAME | TOMBI_USER_CONFIG_FILENAME) => {
            match crate::config::from_str(&config_text, config_path) {
                Ok(tombi_config) => Ok(Some(tombi_config)),
                Err(error) => {
                    tracing::error!(?error);
                    Err(tombi_config::Error::ConfigFileParseFailed {
                        config_path: config_path.to_owned(),
                    })
                }
            }
        }
        Some(PYPROJECT_FILENAME) => {
            let Ok(pyproject_toml) = PyProjectToml::from_str(&config_text, config_path) else {
                return Err(tombi_config::Error::ConfigFileParseFailed {
                    config_path: config_path.to_owned(),
                });
            };
            if let Some(Tool {
                tombi: Some(tombi_config),
            }) = pyproject_toml.tool
            {
                Ok(Some(tombi_config))
            } else {
                Ok(None)
            }
        }
        _ => Err(tombi_config::Error::ConfigFileUnsupported {
            config_path: config_path.to_owned(),
        }),
    }
}

pub fn try_from_url(config_url: &url::Url) -> Result<Option<Config>, tombi_config::Error> {
    match config_url.scheme() {
        "file" => {
            let config_path = url_to_file_path(config_url).map_err(|_| {
                tombi_config::Error::ConfigUrlParseFailed {
                    config_url: config_url.clone(),
                }
            })?;
            try_from_path(config_path)
        }
        _ => Err(tombi_config::Error::ConfigUrlUnsupported {
            config_url: config_url.clone(),
        }),
    }
}

/// Load the config from the current directory.
pub fn load_with_path() -> Result<(Config, Option<std::path::PathBuf>), tombi_config::Error> {
    let mut current_dir = std::env::current_dir().unwrap();
    loop {
        let config_path = current_dir.join(TOMBI_CONFIG_FILENAME);
        if config_path.exists() {
            tracing::debug!("\"{}\" found at {:?}", TOMBI_CONFIG_FILENAME, &config_path);

            let Some(config) = try_from_path(&config_path)? else {
                unreachable!("tombi.toml should always be parsed successfully.");
            };

            return Ok((config, Some(config_path)));
        }

        let pyproject_toml_path = current_dir.join(PYPROJECT_FILENAME);
        if pyproject_toml_path.exists() {
            tracing::debug!(
                "\"{}\" found at {:?}",
                PYPROJECT_FILENAME,
                pyproject_toml_path
            );

            match try_from_path(&pyproject_toml_path)? {
                Some(config) => return Ok((config, Some(pyproject_toml_path))),
                None => {
                    tracing::debug!("No [tool.tombi] found in {:?}", &pyproject_toml_path);
                }
            };
        }

        if !current_dir.pop() {
            break;
        }
    }

    if let Some(user_config_path) = user_tombi_config_path() {
        tracing::debug!("user config.toml found at {:?}", &user_config_path);
        let Some(config) = try_from_path(&user_config_path)? else {
            unreachable!("user config.toml should always be parsed successfully.");
        };
        Ok((config, Some(user_config_path)))
    } else {
        tracing::debug!("config file not found, use default config");

        Ok((Config::default(), None))
    }
}

pub fn load() -> Result<Config, tombi_config::Error> {
    let (config, _) = load_with_path()?;
    Ok(config)
}

pub fn user_tombi_config_path() -> Option<std::path::PathBuf> {
    // 1. $XDG_CONFIG_HOME/tombi/config.toml
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        let mut config_path = std::path::PathBuf::from(xdg_config_home);
        config_path.push("tombi");
        config_path.push(TOMBI_USER_CONFIG_FILENAME);
        if config_path.exists() {
            return Some(config_path);
        }
    }

    if let Some(home_dir) = dirs::home_dir() {
        // 2. ~/.config/tombi/config.toml
        let mut config_path = home_dir.clone();
        config_path.push(".config");
        config_path.push("tombi");
        config_path.push(TOMBI_USER_CONFIG_FILENAME);
        if config_path.exists() {
            return Some(config_path);
        }

        #[cfg(target_os = "macos")]
        {
            // 3. ~/Library/Application Support/tombi/config.toml
            let mut path = home_dir;
            path.push("Library");
            path.push("Application Support");
            path.push("tombi");
            path.push(TOMBI_USER_CONFIG_FILENAME);
            if path.exists() {
                return Some(path);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 3. %APPDATA%\tombi\config.toml
        if let Ok(appdata) = std::env::var("APPDATA") {
            let mut config_path = std::path::PathBuf::from(appdata);
            config_path.push("tombi");
            config_path.push(TOMBI_USER_CONFIG_FILENAME);
            if config_path.exists() {
                return Some(config_path);
            }
        }
    }

    None
}
