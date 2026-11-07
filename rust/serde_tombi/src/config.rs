use tombi_config::{
    CONFIG_TOML_FILENAME, Config, ConfigLevel, DOT_TOMBI_TOML_FILENAME, PYPROJECT_TOML_FILENAME,
    TOMBI_TOML_FILENAME,
};
#[cfg(test)]
use tombi_toml_version::TomlVersion;

mod inheritance;
pub use inheritance::ConfigSourceCache;

/// Parse and resolve a Tombi config document using asynchronous local and HTTP reads.
pub async fn from_str(
    toml_text: &str,
    config_path: &std::path::Path,
) -> Result<Config, tombi_config::Error> {
    inheritance::from_str(toml_text, config_path).await
}

#[cfg(not(target_family = "wasm"))]
fn runtime_error(reason: std::string::String) -> tombi_config::Error {
    tombi_config::Error::ConfigExtendsParseFailed {
        config_source: "config resolver".to_owned(),
        reason,
    }
}

#[cfg(not(target_family = "wasm"))]
fn run_async_in_thread<T, F, Fut>(run: F) -> Result<T, tombi_config::Error>
where
    T: Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T, tombi_config::Error>> + 'static,
{
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| runtime_error(error.to_string()))?;
        runtime.block_on(run())
    })
    .join()
    .map_err(|_| runtime_error("config resolver thread panicked".to_owned()))?
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ConfigLoadOptions {
    pub offline: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigLoadReport {
    pub config: Config,
    pub local_dependencies: Vec<std::path::PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ConfigCandidate {
    pub path: std::path::PathBuf,
    pub level: ConfigLevel,
}

pub fn config_candidates(search_dir: Option<std::path::PathBuf>) -> Vec<ConfigCandidate> {
    let mut candidates = Vec::new();
    if let Some(mut current_dir) = search_dir {
        loop {
            for path in [
                current_dir.join(DOT_TOMBI_TOML_FILENAME),
                current_dir.join(TOMBI_TOML_FILENAME),
                current_dir.join(".config").join(TOMBI_TOML_FILENAME),
                current_dir.join(PYPROJECT_TOML_FILENAME),
            ] {
                candidates.push(ConfigCandidate {
                    path,
                    level: ConfigLevel::Project,
                });
            }
            if !current_dir.pop() {
                break;
            }
        }
    }
    if let Some((path, level)) = get_user_or_system_tombi_config_path_and_level() {
        candidates.push(ConfigCandidate { path, level });
    }
    candidates
}

pub fn try_from_path_sync<P: AsRef<std::path::Path>>(
    config_path: P,
) -> Result<Option<Config>, tombi_config::Error> {
    let config_path = config_path.as_ref().to_owned();
    validate_config_path(&config_path)?;
    #[cfg(not(target_family = "wasm"))]
    {
        run_async_in_thread(move || async move { try_from_path(config_path).await })
    }
    #[cfg(target_family = "wasm")]
    {
        inheritance::try_from_path_sync(&config_path)
    }
}

fn validate_config_path(config_path: &std::path::Path) -> Result<(), tombi_config::Error> {
    match config_path.file_name().and_then(|name| name.to_str()) {
        Some(
            DOT_TOMBI_TOML_FILENAME
            | TOMBI_TOML_FILENAME
            | CONFIG_TOML_FILENAME
            | PYPROJECT_TOML_FILENAME,
        ) => Ok(()),
        _ => Err(tombi_config::Error::ConfigFileUnsupported {
            config_path: config_path.to_owned(),
        }),
    }
}

pub async fn try_from_path<P: AsRef<std::path::Path>>(
    config_path: P,
) -> Result<Option<Config>, tombi_config::Error> {
    Ok(
        try_from_path_with_options(config_path, ConfigLoadOptions::default())
            .await?
            .map(|report| report.config),
    )
}

pub async fn try_from_path_with_options<P: AsRef<std::path::Path>>(
    config_path: P,
    options: ConfigLoadOptions,
) -> Result<Option<ConfigLoadReport>, tombi_config::Error> {
    ConfigSourceCache::default()
        .try_from_path_with_options(config_path, options)
        .await
}

pub fn load_with_path_and_level(
    search_dir: Option<std::path::PathBuf>,
) -> Result<(Config, Option<std::path::PathBuf>, ConfigLevel), tombi_config::Error> {
    #[cfg(not(target_family = "wasm"))]
    {
        run_async_in_thread(move || async move { load_with_path_and_level_async(search_dir).await })
    }
    #[cfg(target_family = "wasm")]
    {
        for candidate in config_candidates(search_dir) {
            if !tombi_fs::is_file(&candidate.path) {
                continue;
            }
            match try_from_path_sync(&candidate.path) {
                Ok(Some(config)) => return Ok((config, Some(candidate.path), candidate.level)),
                Ok(None) => {}
                Err(error @ tombi_config::Error::ConfigAsyncRequired { .. }) => return Err(error),
                Err(error) => log::warn!("failed to load config {:?}: {error}", candidate.path),
            }
        }
        Ok((Config::default(), None, ConfigLevel::Default))
    }
}

pub async fn load_with_path_and_level_async(
    search_dir: Option<std::path::PathBuf>,
) -> Result<(Config, Option<std::path::PathBuf>, ConfigLevel), tombi_config::Error> {
    load_with_path_and_level_with_options(search_dir, ConfigLoadOptions::default()).await
}

pub async fn load_with_path_and_level_with_options(
    search_dir: Option<std::path::PathBuf>,
    options: ConfigLoadOptions,
) -> Result<(Config, Option<std::path::PathBuf>, ConfigLevel), tombi_config::Error> {
    for candidate in config_candidates(search_dir) {
        if !tombi_fs::is_file(&candidate.path) {
            continue;
        }
        match try_from_path_with_options(&candidate.path, options).await {
            Ok(Some(report)) => return Ok((report.config, Some(candidate.path), candidate.level)),
            Ok(None) => {}
            Err(error @ tombi_config::Error::ConfigOffline { .. }) => return Err(error),
            Err(error) => log::warn!("failed to load config {:?}: {error}", candidate.path),
        }
    }
    Ok((Config::default(), None, ConfigLevel::Default))
}

pub async fn load_with_path_async(
    search_dir: Option<std::path::PathBuf>,
) -> Result<(Config, Option<std::path::PathBuf>), tombi_config::Error> {
    let (config, config_path, _) = load_with_path_and_level_async(search_dir).await?;
    Ok((config, config_path))
}

pub async fn load_async(
    search_dir: Option<std::path::PathBuf>,
) -> Result<Config, tombi_config::Error> {
    let (config, _, _) = load_with_path_and_level_async(search_dir).await?;
    Ok(config)
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
