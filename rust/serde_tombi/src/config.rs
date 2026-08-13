use tombi_ast::AstNode;
use tombi_config::{
    CONFIG_TOML_FILENAME, Config, ConfigLevel, DOT_TOMBI_TOML_FILENAME, PYPROJECT_TOML_FILENAME,
    TOMBI_CONFIG_TOML_VERSION, TOMBI_TOML_FILENAME, TomlVersion,
};

/// Parse the TOML text into a `Config` struct.
///
/// When executing [crate::from_str_async], it is necessary to obtain the Config to determine the TOML version.
/// If [crate::from_str_async] is used to parse the Config, it will cause a stack overflow due to circular references.
/// Therefore, [crate::config::from_str], which does not use schema_store and is not async, is called to prevent stack overflow.
///
/// This parser uses the TOML version required by Tombi configuration files and
/// avoids recursively loading configuration while deserializing `Config`.
pub fn from_str(
    toml_text: &str,
    config_path: &std::path::Path,
) -> Result<Config, crate::de::Error> {
    let deserializer = crate::Deserializer::builder()
        .config_path(config_path)
        .build();

    let parsed = tombi_parser::parse(toml_text);
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

        let parsed = tombi_parser::parse(toml_text);
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

#[inline]
fn config_file_parse_error(config_path: &std::path::Path) -> tombi_config::Error {
    let error = tombi_config::Error::ConfigFileParseFailed {
        config_path: config_path.to_owned(),
    };

    log::warn!("{}", error);

    error
}

#[inline]
fn config_file_read_error(config_path: &std::path::Path) -> tombi_config::Error {
    let error = tombi_config::Error::ConfigFileReadFailed {
        config_path: config_path.to_owned(),
    };

    log::warn!("{}", error);

    error
}

pub fn try_from_path<P: AsRef<std::path::Path>>(
    config_path: P,
) -> Result<Option<Config>, tombi_config::Error> {
    let config_path = config_path.as_ref();

    if !tombi_fs::is_file(config_path) {
        return Err(tombi_config::Error::ConfigFileNotFound {
            config_path: config_path.to_owned(),
        });
    }

    let config_text =
        tombi_fs::read_to_string(config_path).map_err(|_| config_file_read_error(config_path))?;

    match config_path.file_name().and_then(|name| name.to_str()) {
        Some(DOT_TOMBI_TOML_FILENAME | TOMBI_TOML_FILENAME | CONFIG_TOML_FILENAME) => {
            crate::config::from_str(&config_text, config_path)
                .map(Some)
                .map_err(|_| config_file_parse_error(config_path))
        }
        Some(PYPROJECT_TOML_FILENAME) => {
            let pyproject_toml = PyProjectToml::from_str(&config_text, config_path)
                .map_err(|_| config_file_parse_error(config_path))?;
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

pub fn load_with_path_and_level(
    search_dir: Option<std::path::PathBuf>,
) -> Result<(Config, Option<std::path::PathBuf>, ConfigLevel), tombi_config::Error> {
    if let Some(mut current_dir) = search_dir {
        loop {
            for config_path in [
                current_dir.join(DOT_TOMBI_TOML_FILENAME),
                current_dir.join(TOMBI_TOML_FILENAME),
                current_dir.join(".config").join(TOMBI_TOML_FILENAME),
            ] {
                log::trace!("checking config file at {:?}", config_path);
                if tombi_fs::is_file(&config_path) {
                    log::debug!("project config found at {:?}", config_path);

                    match try_from_path(&config_path) {
                        Ok(Some(config)) => {
                            return Ok((config, Some(config_path), ConfigLevel::Project));
                        }
                        Ok(None) => {
                            unreachable!(
                                "project config should always be parsed successfully: {:?}",
                                config_path
                            );
                        }
                        Err(_) => {}
                    }
                }
            }

            let pyproject_toml_path = current_dir.join(PYPROJECT_TOML_FILENAME);
            log::trace!("checking pyproject.toml file at {:?}", pyproject_toml_path);
            if tombi_fs::is_file(&pyproject_toml_path) {
                log::debug!(
                    "\"{}\" found at {:?}",
                    PYPROJECT_TOML_FILENAME,
                    pyproject_toml_path
                );

                match try_from_path(&pyproject_toml_path).ok().flatten() {
                    Some(config) => {
                        return Ok((config, Some(pyproject_toml_path), ConfigLevel::Project));
                    }
                    None => {
                        log::debug!("no [tool.tombi] found in {:?}", pyproject_toml_path);
                    }
                };
            }

            if !current_dir.pop() {
                break;
            }
        }
    }

    if let Some((user_config_path, config_level)) = get_user_or_system_tombi_config_path_and_level()
    {
        log::debug!("{CONFIG_TOML_FILENAME} found at {:?}", user_config_path);
        match try_from_path(&user_config_path).ok().flatten() {
            Some(config) => {
                return Ok((config, Some(user_config_path), config_level));
            }
            None => {
                unreachable!("{CONFIG_TOML_FILENAME} should always be parsed successfully.");
            }
        }
    }

    log::debug!("config file not found, use default config");

    Ok((Config::default(), None, ConfigLevel::Default))
}

#[inline]
pub fn load_with_path(
    search_dir: Option<std::path::PathBuf>,
) -> Result<(Config, Option<std::path::PathBuf>), tombi_config::Error> {
    let (config, config_path, _) = load_with_path_and_level(search_dir)?;
    Ok((config, config_path))
}

#[inline]
pub fn load(search_dir: Option<std::path::PathBuf>) -> Result<Config, tombi_config::Error> {
    let (config, _, _) = load_with_path_and_level(search_dir)?;
    Ok(config)
}

fn get_user_or_system_tombi_config_path_and_level() -> Option<(std::path::PathBuf, ConfigLevel)> {
    // 1. $XDG_CONFIG_HOME/tombi/config.toml
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        let mut config_path = std::path::PathBuf::from(xdg_config_home);
        config_path.push("tombi");
        config_path.push(CONFIG_TOML_FILENAME);
        if tombi_fs::is_file(&config_path) {
            return Some((config_path, ConfigLevel::User));
        }
    }

    if let Some(home_dir) = dirs::home_dir() {
        // 2. ~/.config/tombi/config.toml
        let mut config_path = home_dir.clone();
        config_path.push(".config");
        config_path.push("tombi");
        config_path.push(CONFIG_TOML_FILENAME);
        if tombi_fs::is_file(&config_path) {
            return Some((config_path, ConfigLevel::User));
        }

        #[cfg(target_os = "macos")]
        {
            // 3. ~/Library/Application Support/tombi/config.toml
            let mut config_path = home_dir;
            config_path.push("Library");
            config_path.push("Application Support");
            config_path.push("tombi");
            config_path.push(CONFIG_TOML_FILENAME);
            if tombi_fs::is_file(&config_path) {
                return Some((config_path, ConfigLevel::User));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 3. %APPDATA%\tombi\config.toml
        if let Ok(appdata) = std::env::var("APPDATA") {
            let mut config_path = std::path::PathBuf::from(appdata);
            config_path.push("tombi");
            config_path.push(CONFIG_TOML_FILENAME);
            if tombi_fs::is_file(&config_path) {
                return Some((config_path, ConfigLevel::User));
            }
        }
    }

    // 4. /etc/tombi/config.toml
    let mut config_path = std::path::PathBuf::from("/etc/tombi");
    config_path.push(CONFIG_TOML_FILENAME);
    if tombi_fs::is_file(&config_path) {
        return Some((config_path, ConfigLevel::System));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_file(path: &std::path::Path, contents: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn temp_test_dir(test_name: &str) -> TempDir {
        tempfile::Builder::new()
            .prefix(test_name)
            .tempdir()
            .unwrap()
    }

    #[test]
    fn loads_next_project_config_after_parse_error() {
        let temp_dir = temp_test_dir("config-parse-error-fallback");
        let nested_dir = temp_dir.path().join("workspace/nested");

        std::fs::create_dir_all(&nested_dir).unwrap();
        write_file(
            &temp_dir.path().join(DOT_TOMBI_TOML_FILENAME),
            "toml-version = ",
        );
        write_file(
            &temp_dir.path().join(TOMBI_TOML_FILENAME),
            "toml-version = \"v1.0.0\"\n",
        );

        let (config, config_path, config_level) =
            load_with_path_and_level(Some(nested_dir)).unwrap();

        assert_eq!(config.toml_version, Some(TomlVersion::V1_0_0));
        assert_eq!(config_path, Some(temp_dir.path().join(TOMBI_TOML_FILENAME)));
        assert_eq!(config_level, ConfigLevel::Project);
    }

    #[test]
    fn loads_project_config_from_dot_config_tombi_toml_before_pyproject() {
        let temp_dir = temp_test_dir("dot-config-tombi");
        let nested_dir = temp_dir.path().join("workspace/nested");

        std::fs::create_dir_all(&nested_dir).unwrap();
        write_file(
            &temp_dir.path().join(".config/tombi.toml"),
            "toml-version = \"v1.1.0\"\n",
        );
        write_file(
            &temp_dir.path().join("pyproject.toml"),
            "[tool.tombi]\ntoml-version = \"v1.0.0\"\n",
        );

        let (config, config_path, config_level) =
            load_with_path_and_level(Some(nested_dir)).unwrap();

        assert_eq!(config.toml_version, Some(TomlVersion::V1_1_0));
        assert_eq!(
            config_path,
            Some(temp_dir.path().join(".config/tombi.toml"))
        );
        assert_eq!(config_level, ConfigLevel::Project);
    }

    #[test]
    fn ignores_dot_tombi_toml_inside_dot_config() {
        let temp_dir = temp_test_dir("ignore-dot-config-dot-tombi");
        let nested_dir = temp_dir.path().join("workspace/nested");

        std::fs::create_dir_all(&nested_dir).unwrap();
        write_file(
            &temp_dir.path().join(".config/.tombi.toml"),
            "toml-version = \"v1.1.0\"\n",
        );
        write_file(
            &temp_dir.path().join(".config/tombi.toml"),
            "toml-version = \"v1.0.0\"\n",
        );

        let (config, config_path, config_level) =
            load_with_path_and_level(Some(nested_dir)).unwrap();

        assert_eq!(config.toml_version, Some(TomlVersion::V1_0_0));
        assert_eq!(
            config_path,
            Some(temp_dir.path().join(".config/tombi.toml"))
        );
        assert_eq!(config_level, ConfigLevel::Project);
    }
}
