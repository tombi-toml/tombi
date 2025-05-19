use tombi_ast::AstNode;
use tombi_config::{
    Config, TomlVersion, CONFIG_FILENAME, PYPROJECT_FILENAME, TOMBI_CONFIG_TOML_VERSION,
};
use tombi_wasm_compat::url::url_to_file_path;

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
        return Err(crate::de::Error::Parser(parsed.errors));
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
            return Err(crate::de::Error::Parser(parsed.errors));
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
        Some(CONFIG_FILENAME) => match crate::config::from_str(&config_text, config_path) {
            Ok(tombi_config) => Ok(Some(tombi_config)),
            Err(error) => {
                tracing::error!(?error);
                Err(tombi_config::Error::ConfigFileParseFailed {
                    config_path: config_path.to_owned(),
                })
            }
        },
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

pub fn try_from_url(config_url: url::Url) -> Result<Option<Config>, tombi_config::Error> {
    match config_url.scheme() {
        "file" => {
            let config_path = url_to_file_path(&config_url)
                .map_err(|_| tombi_config::Error::ConfigUrlParseFailed { config_url })?;
            try_from_path(config_path)
        }
        _ => Err(tombi_config::Error::ConfigUrlUnsupported { config_url }),
    }
}

/// Load the config from the current directory.
pub fn load_with_path() -> Result<(Config, Option<std::path::PathBuf>), tombi_config::Error> {
    let mut current_dir = std::env::current_dir().unwrap();
    loop {
        let config_path = current_dir.join(CONFIG_FILENAME);
        if config_path.exists() {
            tracing::debug!("\"{}\" found at {:?}", CONFIG_FILENAME, &config_path);

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
                    tracing::debug!("No [tool.tombi] found in {:?}", &config_path);
                }
            };
        }

        if !current_dir.pop() {
            break;
        }
    }

    tracing::debug!("config file not found, use default config");

    Ok((Config::default(), None))
}

pub fn load() -> Result<Config, tombi_config::Error> {
    let (config, _) = load_with_path()?;
    Ok(config)
}
