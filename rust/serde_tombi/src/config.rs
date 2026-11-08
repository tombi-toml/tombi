use itertools::Itertools;
use tombi_config::{
    CONFIG_TOML_FILENAME, Config, ConfigLevel, DOT_TOMBI_TOML_FILENAME, PYPROJECT_TOML_FILENAME,
    TOMBI_TOML_FILENAME,
};
#[cfg(test)]
use tombi_toml_version::TomlVersion;

use tombi_schema_store::CachedHttpClient;

mod inheritance;
pub use inheritance::ConfigSourceCache;

/// Parse and resolve a Tombi config document using asynchronous local and HTTP reads.
pub async fn from_str(
    toml_text: &str,
    config_path: &std::path::Path,
    http_client: &CachedHttpClient,
) -> Result<Config, tombi_config::Error> {
    inheritance::from_str(toml_text, config_path, http_client).await
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
            let misplaced = misplaced_dot_config_tombi_toml(&current_dir);
            if tombi_fs::is_file(&misplaced) {
                log::warn!(
                    "\"{DOT_TOMBI_TOML_FILENAME}\" in the \".config\" directory is ignored. Rename it to \"{TOMBI_TOML_FILENAME}\": {misplaced:?}"
                );
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

/// `.config/.tombi.toml` is never loaded; only `.config/tombi.toml` is supported.
fn misplaced_dot_config_tombi_toml(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join(".config").join(DOT_TOMBI_TOML_FILENAME)
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
    http_client: &CachedHttpClient,
) -> Result<Option<Config>, tombi_config::Error> {
    Ok(ConfigSourceCache::default()
        .try_from_path(config_path, http_client)
        .await?
        .map(|report| report.config))
}

pub async fn load_with_path_and_level(
    search_dir: Option<std::path::PathBuf>,
    http_client: &CachedHttpClient,
) -> Result<(Config, Option<std::path::PathBuf>, ConfigLevel), tombi_config::Error> {
    for candidate in config_candidates(search_dir) {
        if !tombi_fs::is_file(&candidate.path) {
            continue;
        }
        match try_from_path(&candidate.path, http_client).await {
            Ok(Some(config)) => return Ok((config, Some(candidate.path), candidate.level)),
            Ok(None) => {}
            Err(error) => log::error!("failed to load config {:?}: {error}", candidate.path),
        }
    }
    Ok((Config::default(), None, ConfigLevel::Default))
}

pub async fn load_with_path(
    search_dir: Option<std::path::PathBuf>,
    http_client: &CachedHttpClient,
) -> Result<(Config, Option<std::path::PathBuf>), tombi_config::Error> {
    let (config, config_path, _) = load_with_path_and_level(search_dir, http_client).await?;
    Ok((config, config_path))
}

pub async fn load(
    search_dir: Option<std::path::PathBuf>,
    http_client: &CachedHttpClient,
) -> Result<Config, tombi_config::Error> {
    let (config, _, _) = load_with_path_and_level(search_dir, http_client).await?;
    Ok(config)
}

/// `tombi.toml` next to the given user `config.toml` paths is never loaded at user level.
fn misplaced_tombi_tomls(
    user_config_paths: &[std::path::PathBuf],
) -> impl Iterator<Item = std::path::PathBuf> + '_ {
    user_config_paths
        .iter()
        .unique()
        .map(|path| path.with_file_name(TOMBI_TOML_FILENAME))
        .filter(|path| tombi_fs::is_file(path))
}

fn get_user_or_system_tombi_config_path_and_level() -> Option<(std::path::PathBuf, ConfigLevel)> {
    let mut user_config_paths = Vec::new();

    // 1. $XDG_CONFIG_HOME/tombi/config.toml
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        let mut config_path = std::path::PathBuf::from(xdg_config_home);
        config_path.push("tombi");
        config_path.push(CONFIG_TOML_FILENAME);
        if tombi_fs::is_file(&config_path) {
            return Some((config_path, ConfigLevel::User));
        }
        user_config_paths.push(config_path.clone());
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
        user_config_paths.push(config_path.clone());

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
            user_config_paths.push(config_path.clone());
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
            user_config_paths.push(config_path.clone());
        }
    }

    for path in misplaced_tombi_tomls(&user_config_paths) {
        log::warn!(
            "\"{TOMBI_TOML_FILENAME}\" in the user config directory is ignored. Rename it to \"{CONFIG_TOML_FILENAME}\": {path:?}"
        );
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

    #[tokio::test]
    async fn loads_next_project_config_after_parse_error() {
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
            load_with_path_and_level(Some(nested_dir), offline_client().as_ref())
                .await
                .unwrap();

        assert_eq!(config.toml_version, Some(TomlVersion::V1_0_0));
        assert_eq!(config_path, Some(temp_dir.path().join(TOMBI_TOML_FILENAME)));
        assert_eq!(config_level, ConfigLevel::Project);
    }

    #[tokio::test]
    async fn loads_project_config_from_dot_config_tombi_toml_before_pyproject() {
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
            load_with_path_and_level(Some(nested_dir), offline_client().as_ref())
                .await
                .unwrap();

        assert_eq!(config.toml_version, Some(TomlVersion::V1_1_0));
        assert_eq!(
            config_path,
            Some(temp_dir.path().join(".config/tombi.toml"))
        );
        assert_eq!(config_level, ConfigLevel::Project);
    }

    #[tokio::test]
    async fn ignores_dot_tombi_toml_inside_dot_config() {
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
            load_with_path_and_level(Some(nested_dir), offline_client().as_ref())
                .await
                .unwrap();

        assert_eq!(config.toml_version, Some(TomlVersion::V1_0_0));
        assert_eq!(
            config_path,
            Some(temp_dir.path().join(".config/tombi.toml"))
        );
        assert_eq!(config_level, ConfigLevel::Project);
    }

    #[tokio::test]
    async fn extends_accepts_dot_tombi_toml() {
        let temp_dir = temp_test_dir("extends-dot-tombi");
        write_file(
            &temp_dir.path().join("shared/.tombi.toml"),
            "toml-version = \"v1.0.0\"\n",
        );
        let config_path = temp_dir.path().join("app/tombi.toml");
        write_file(&config_path, "extends = [\"../shared/.tombi.toml\"]\n");

        let config = try_from_path(&config_path, offline_client().as_ref())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(config.toml_version, Some(TomlVersion::V1_0_0));
    }

    fn offline_client() -> std::sync::Arc<CachedHttpClient> {
        tombi_schema_store::http_client(tombi_schema_store::HttpClientOptions {
            offline: true,
            cache: None,
        })
    }

    #[tokio::test]
    async fn extends_rejects_files_not_named_tombi_toml() {
        let temp_dir = temp_test_dir("extends-other-name");
        write_file(
            &temp_dir.path().join("shared/base.toml"),
            "toml-version = \"v1.0.0\"\n",
        );
        let config_path = temp_dir.path().join("app/tombi.toml");
        write_file(&config_path, "extends = [\"../shared/base.toml\"]\n");

        let error = try_from_path(&config_path, offline_client().as_ref())
            .await
            .unwrap_err();

        assert!(
            matches!(error, tombi_config::Error::ConfigExtendsInvalid { .. }),
            "{error}"
        );
    }

    #[tokio::test]
    async fn extends_rejects_string_form() {
        let temp_dir = temp_test_dir("extends-string-form");
        write_file(
            &temp_dir.path().join("shared/tombi.toml"),
            "toml-version = \"v1.0.0\"\n",
        );
        let config_path = temp_dir.path().join("app/tombi.toml");
        write_file(&config_path, "extends = \"../shared/tombi.toml\"\n");

        let error = try_from_path(&config_path, offline_client().as_ref())
            .await
            .unwrap_err();

        assert!(
            matches!(
                &error,
                tombi_config::Error::ConfigExtendsInvalid { reason, .. }
                    if reason == "extends must be an array of strings"
            ),
            "{error}"
        );
    }

    #[tokio::test]
    async fn extends_replaces_nested_extension_feature_forms() {
        let temp_dir = temp_test_dir("extends-extension-feature-form");
        write_file(
            &temp_dir.path().join("shared/tombi.toml"),
            "[extensions.\"tombi-toml/cargo\".lsp.hover]\nenabled = true\n",
        );
        let config_path = temp_dir.path().join("app/tombi.toml");
        write_file(
            &config_path,
            "extends = [\"../shared/tombi.toml\"]\n\n[extensions.\"tombi-toml/cargo\".lsp]\nenabled = false\n",
        );

        let config = try_from_path(&config_path, offline_client().as_ref())
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(
            config.cargo_extension_features().unwrap().lsp(),
            Some(tombi_config::CargoLspFeatures::Enabled(_))
        ));
    }

    #[tokio::test]
    async fn extends_rejects_tombi_toml_only_in_query_or_fragment() {
        let temp_dir = temp_test_dir("extends-query-bypass");
        write_file(&temp_dir.path().join("shared/tombi.toml"), "");
        write_file(&temp_dir.path().join("shared/base.toml"), "");
        let file_url = |suffix: &str| {
            format!(
                "file://{}/shared/{suffix}",
                temp_dir.path().display().to_string().trim_end_matches('/')
            )
        };

        for reference in [
            "../shared/base.toml?x=tombi.toml".to_owned(),
            "../shared/base.toml#tombi.toml".to_owned(),
            "https://example.com/base.toml?file=tombi.toml".to_owned(),
            "https://example.com/base.toml#/tombi.toml".to_owned(),
            "https://example.com/tombi.toml/".to_owned(),
            file_url("base.toml?x=tombi.toml"),
            file_url("tombi.toml?x=1"),
        ] {
            let config_path = temp_dir.path().join("app/tombi.toml");
            write_file(&config_path, &format!("extends = [\"{reference}\"]\n"));

            let error = try_from_path(&config_path, offline_client().as_ref())
                .await
                .unwrap_err();

            assert!(
                matches!(error, tombi_config::Error::ConfigExtendsInvalid { .. }),
                "{reference}: {error}"
            );
        }
    }

    static WARNINGS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

    struct WarningCollector;

    impl log::Log for WarningCollector {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= log::Level::Warn
        }

        fn log(&self, record: &log::Record) {
            if record.level() == log::Level::Warn {
                WARNINGS.lock().unwrap().push(record.args().to_string());
            }
        }

        fn flush(&self) {}
    }

    fn warnings_mentioning(root: &std::path::Path, search_dir: &std::path::Path) -> Vec<String> {
        // Ignore the error: another test may have installed the collector already.
        let _ = log::set_logger(&WarningCollector);
        log::set_max_level(log::LevelFilter::Warn);

        config_candidates(Some(search_dir.to_owned()));
        let root = root.to_string_lossy();
        WARNINGS
            .lock()
            .unwrap()
            .iter()
            .filter(|message| message.contains(root.as_ref()))
            .cloned()
            .collect()
    }

    #[test]
    fn warns_on_misplaced_dot_config_tombi_toml_in_ancestor() {
        let temp_dir = temp_test_dir("misplaced-dot-config-tombi-toml");
        let nested = temp_dir.path().join("nested");
        write_file(&nested.join("pyproject.toml"), "");
        write_file(
            &temp_dir
                .path()
                .join(".config")
                .join(DOT_TOMBI_TOML_FILENAME),
            "",
        );

        let warnings = warnings_mentioning(temp_dir.path(), &nested);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains("Rename it to \"tombi.toml\""));
    }

    #[test]
    fn does_not_warn_without_misplaced_dot_config_tombi_toml() {
        let temp_dir = temp_test_dir("no-misplaced-dot-config-tombi-toml");
        write_file(
            &temp_dir.path().join(".config").join(TOMBI_TOML_FILENAME),
            "",
        );

        assert!(warnings_mentioning(temp_dir.path(), temp_dir.path()).is_empty());
    }

    #[test]
    fn detects_tombi_toml_next_to_user_config_toml_path() {
        let temp_dir = temp_test_dir("misplaced-user-tombi-toml");
        let misplaced = temp_dir.path().join("misplaced");
        let empty = temp_dir.path().join("empty");
        write_file(&misplaced.join(TOMBI_TOML_FILENAME), "");

        assert_eq!(
            misplaced_tombi_tomls(&[
                misplaced.join(CONFIG_TOML_FILENAME),
                misplaced.join(CONFIG_TOML_FILENAME),
                empty.join(CONFIG_TOML_FILENAME)
            ])
            .collect::<Vec<_>>(),
            vec![misplaced.join(TOMBI_TOML_FILENAME)]
        );
    }
}
