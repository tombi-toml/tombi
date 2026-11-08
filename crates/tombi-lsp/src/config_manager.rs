use std::path::{Path, PathBuf};
use std::sync::Arc;

use tombi_config::Config;
use tombi_schema_store::{SchemaStore, SchemaUri};

/// Source type for ConfigSchemaStore without a specific config file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultConfigSource {
    /// Default configuration
    Default,
    /// Configuration from editor settings via workspace/didChangeConfiguration
    Editor,
}

/// Holds a Config and its associated SchemaStore
#[derive(Debug, Clone)]
pub struct ConfigSchemaStore {
    pub config: Config,
    pub config_path: Option<PathBuf>,
    pub schema_store: SchemaStore,
}

impl ConfigSchemaStore {
    pub fn new(config: Config, config_path: Option<PathBuf>, schema_store: SchemaStore) -> Self {
        Self {
            config,
            config_path,
            schema_store,
        }
    }
}

/// A config with its store, and the local config files it is built from (itself included).
#[derive(Debug)]
struct LoadedConfig {
    store: ConfigSchemaStore,
    local_dependencies: Vec<PathBuf>,
}

/// Manages configuration files and their associated schema stores for different source files
///
/// Lock order: `config_source_cache`, then `associated_schemas`, then the store maps.
#[derive(Debug)]
pub struct ConfigManager {
    /// Maps source file paths to their associated config file paths
    source_config_paths: Arc<tokio::sync::RwLock<tombi_hashmap::HashMap<PathBuf, PathBuf>>>,
    /// Maps config file paths to their loaded config
    config_schema_stores: Arc<tokio::sync::RwLock<tombi_hashmap::HashMap<PathBuf, LoadedConfig>>>,
    /// Serializes config resolution, which is the only part that reads and fetches configs.
    config_source_cache: tokio::sync::Mutex<serde_tombi::config::ConfigSourceCache>,
    /// Default ConfigSchemaStore when no config file is found
    /// Contains the source type (Default or Editor) and the ConfigSchemaStore
    default_config_schema_store:
        Arc<tokio::sync::RwLock<Option<(DefaultConfigSource, ConfigSchemaStore)>>>,

    backend_options: crate::backend::Options,
    /// Reads remote `extends` configs through the shared on-disk cache.
    http_client: Arc<tombi_schema_store::CachedHttpClient>,
    /// Append-only. Holding it keeps `associate_schema` from running, so that a store
    /// built from a snapshot of it is registered without missing an association.
    associated_schemas: Arc<tokio::sync::RwLock<Vec<AssociatedSchema>>>,
}

#[derive(Debug, Clone)]
struct AssociatedSchema {
    schema_uri: SchemaUri,
    file_match: Vec<String>,
    options: tombi_schema_store::AssociateSchemaOptions,
}

impl ConfigManager {
    pub fn new(backend_options: &crate::backend::Options) -> Self {
        Self {
            source_config_paths: Arc::new(tokio::sync::RwLock::new(tombi_hashmap::HashMap::new())),
            config_schema_stores: Arc::new(tokio::sync::RwLock::new(tombi_hashmap::HashMap::new())),
            config_source_cache: tokio::sync::Mutex::new(Default::default()),
            default_config_schema_store: Arc::new(tokio::sync::RwLock::new(None)),
            backend_options: backend_options.clone(),
            http_client: tombi_schema_store::http_client(tombi_schema_store::HttpClientOptions {
                offline: backend_options.offline.unwrap_or_default(),
                cache: Some(backend_options.cache.clone()),
            }),
            associated_schemas: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Forget which config each source file uses, so config discovery runs again.
    pub async fn clear_source_config_paths(&self) {
        self.source_config_paths.write().await.clear();
    }

    /// Get config for a URI
    pub async fn config_schema_store_for_uri(
        &self,
        text_document_uri: &tombi_uri::Uri,
    ) -> ConfigSchemaStore {
        if let Ok(text_document_path) = text_document_uri.to_file_path() {
            self.config_schema_store_for_file(&text_document_path).await
        } else {
            self.default_config_schema_store().await
        }
    }

    pub async fn config_schema_store_for_file(
        &self,
        text_document_path: &Path,
    ) -> ConfigSchemaStore {
        let cached_config_path = self
            .source_config_paths
            .read()
            .await
            .get(text_document_path)
            .cloned();
        if let Some(config_path) = cached_config_path
            && let Some(config) = self.config_schema_stores.read().await.get(&config_path)
        {
            return config.store.clone();
        }

        let mut source_cache = self.config_source_cache.lock().await;
        let store = self
            .config_schema_store_for_directory_locked(
                text_document_path.parent().map(ToOwned::to_owned),
                &mut source_cache,
            )
            .await;
        if let Some(config_path) = &store.config_path {
            self.source_config_paths
                .write()
                .await
                .insert(text_document_path.to_owned(), config_path.clone());
        }
        store
    }

    /// The HTTP client that reads remote configs, honoring offline mode and the cache options.
    pub fn http_client(&self) -> &tombi_schema_store::CachedHttpClient {
        self.http_client.as_ref()
    }

    async fn config_schema_store_for_directory_locked(
        &self,
        directory: Option<PathBuf>,
        source_cache: &mut serde_tombi::config::ConfigSourceCache,
    ) -> ConfigSchemaStore {
        match self
            .try_config_schema_store_for_directory_locked(directory, source_cache)
            .await
        {
            Ok(Some(store)) => store,
            Ok(None) => self.default_config_schema_store().await,
            Err(error) => {
                log::warn!("failed to load schema store: {error}");
                self.default_config_schema_store().await
            }
        }
    }

    async fn try_config_schema_store_for_directory_locked(
        &self,
        directory: Option<PathBuf>,
        source_cache: &mut serde_tombi::config::ConfigSourceCache,
    ) -> Result<Option<ConfigSchemaStore>, tombi_schema_store::Error> {
        for candidate in serde_tombi::config::config_candidates(directory) {
            if let Some(config) = self.config_schema_stores.read().await.get(&candidate.path) {
                return Ok(Some(config.store.clone()));
            }
            if !tombi_fs::is_file(&candidate.path) {
                continue;
            }
            match source_cache
                .try_from_path(&candidate.path, self.http_client.as_ref())
                .await
            {
                Ok(Some(report)) => {
                    return Ok(Some(
                        self.update_config_report(report, &candidate.path).await?,
                    ));
                }
                Ok(None) => {}
                Err(error) => log::warn!("failed to load config {:?}: {error}", candidate.path),
            }
        }
        Ok(None)
    }

    /// Reload a config file and every config that inherits from it via `extends`.
    pub async fn reload_config(
        &self,
        config_path: &Path,
    ) -> Result<bool, tombi_schema_store::Error> {
        let mut source_cache = self.config_source_cache.lock().await;
        source_cache.invalidate_local_paths(&[config_path.to_owned()]);

        let mut config_paths = vec![config_path.to_owned()];
        config_paths.extend(
            dependents_of(&*self.config_schema_stores.read().await, config_path)
                .into_iter()
                .filter(|dependent| dependent != config_path),
        );

        let mut reloaded = false;
        for config_path in config_paths {
            match source_cache
                .try_from_path(&config_path, self.http_client.as_ref())
                .await
            {
                Ok(Some(report)) => {
                    self.update_config_report(report, &config_path).await?;
                    reloaded = true;
                }
                Ok(None) => {}
                Err(error) => {
                    log::warn!("failed to reload config {config_path:?}: {error}");
                }
            }
        }
        Ok(reloaded)
    }

    /// Update a specific config and its path
    pub async fn update_config_with_path(
        &self,
        config: Config,
        config_path: &Path,
    ) -> Result<(), tombi_schema_store::Error> {
        let _source_cache = self.config_source_cache.lock().await;
        self.update_config_report(
            serde_tombi::config::ConfigLoadReport {
                config,
                local_dependencies: vec![
                    absolute_normalized_path(config_path),
                    tombi_fs::canonicalize(config_path)
                        .unwrap_or_else(|_| absolute_normalized_path(config_path)),
                ],
            },
            config_path,
        )
        .await?;
        Ok(())
    }

    async fn update_config_report(
        &self,
        report: serde_tombi::config::ConfigLoadReport,
        config_path: &Path,
    ) -> Result<ConfigSchemaStore, tombi_schema_store::Error> {
        let serde_tombi::config::ConfigLoadReport {
            config,
            local_dependencies,
        } = report;
        let (schema_store, applied, result) =
            self.build_schema_store(&config, Some(config_path)).await;
        result?;
        let store = ConfigSchemaStore::new(config, Some(config_path.to_owned()), schema_store);

        let _associations = self
            .catch_up_associations(&store.schema_store, applied)
            .await;
        self.config_schema_stores.write().await.insert(
            config_path.to_owned(),
            LoadedConfig {
                store: store.clone(),
                local_dependencies,
            },
        );
        Ok(store)
    }

    /// Build a store for `config` from the associations registered so far.
    ///
    /// Returns the store (usable even when loading failed), the number of
    /// associations applied, and the loading result.
    async fn build_schema_store(
        &self,
        config: &Config,
        config_path: Option<&Path>,
    ) -> (SchemaStore, usize, Result<(), tombi_schema_store::Error>) {
        let associated_schemas = self.associated_schemas.read().await.clone();
        let schema_store =
            SchemaStore::new_with_options(schema_store_options(config, &self.backend_options));
        let result = load_schema_store_with_associations(
            &schema_store,
            config,
            config_path,
            &associated_schemas,
        )
        .await;
        (schema_store, associated_schemas.len(), result)
    }

    /// Apply the associations added since `applied` to a freshly built store.
    ///
    /// Keep the returned guard until the store is registered: `associate_schema`
    /// waits for it, so no association can be missed in between.
    async fn catch_up_associations(
        &self,
        schema_store: &SchemaStore,
        applied: usize,
    ) -> tokio::sync::RwLockReadGuard<'_, Vec<AssociatedSchema>> {
        let associated_schemas = self.associated_schemas.read().await;
        load_associations(schema_store, &associated_schemas[applied..]).await;
        associated_schemas
    }

    /// Every store, the default one included.
    async fn all_config_schema_stores(&self) -> Vec<ConfigSchemaStore> {
        let mut stores = self
            .config_schema_stores
            .read()
            .await
            .values()
            .map(|config| config.store.clone())
            .collect::<Vec<_>>();
        if let Some((_, store)) = self.default_config_schema_store.read().await.as_ref() {
            stores.push(store.clone());
        }
        stores
    }

    /// Get the default config
    async fn default_config_schema_store(&self) -> ConfigSchemaStore {
        let (store, result) = self.init_default_config_schema_store().await;
        if let Err(err) = result {
            log::error!("failed to load default schema store: {err}");
        }
        store
    }

    async fn init_default_config_schema_store(
        &self,
    ) -> (ConfigSchemaStore, Result<(), tombi_schema_store::Error>) {
        if let Some((_, store)) = self.default_config_schema_store.read().await.as_ref() {
            return (store.clone(), Ok(()));
        }

        let config = Config::default();
        let (schema_store, applied, result) = self.build_schema_store(&config, None).await;
        let store = ConfigSchemaStore::new(config, None, schema_store);

        let _associations = self
            .catch_up_associations(&store.schema_store, applied)
            .await;
        let mut default_config_schema_store = self.default_config_schema_store.write().await;
        let store = match default_config_schema_store.as_ref() {
            Some((_, existing)) => existing.clone(),
            None => {
                *default_config_schema_store = Some((DefaultConfigSource::Default, store.clone()));
                store
            }
        };
        (store, result)
    }

    /// Get the config path for a specific URI
    pub async fn get_config_path_for_uri(&self, uri: &tombi_uri::Uri) -> Option<PathBuf> {
        if let Ok(path) = uri.to_file_path() {
            let source_config_paths = self.source_config_paths.read().await;
            if let Some(config_path) = source_config_paths.get(&path) {
                return Some(config_path.clone());
            }
        }
        None
    }

    pub async fn update_schema(
        &self,
        schema_uri: SchemaUri,
    ) -> Result<bool, tombi_schema_store::Error> {
        let mut updated = false;
        for config_schema_store in self.all_config_schema_stores().await {
            updated |= config_schema_store
                .schema_store
                .update_schema(schema_uri.clone())
                .await?;
        }
        Ok(updated)
    }

    pub async fn associate_schema(
        &self,
        schema_uri: &SchemaUri,
        file_match: &[String],
        options: &tombi_schema_store::AssociateSchemaOptions,
    ) {
        let mut associated_schemas = self.associated_schemas.write().await;
        let associated_schema = AssociatedSchema {
            schema_uri: schema_uri.clone(),
            file_match: file_match.to_vec(),
            options: options.clone(),
        };
        associated_schemas.push(associated_schema.clone());

        for config_schema_store in self.all_config_schema_stores().await {
            apply_association(&config_schema_store.schema_store, &associated_schema).await;
        }
    }

    pub async fn load(&self) -> Result<(), tombi_schema_store::Error> {
        let mut source_cache = self.config_source_cache.lock().await;
        let has_editor_config = self
            .default_config_schema_store
            .read()
            .await
            .as_ref()
            .is_some_and(|(source, _)| *source == DefaultConfigSource::Editor);
        if !has_editor_config
            && self
                .try_config_schema_store_for_directory_locked(
                    std::env::current_dir().ok(),
                    &mut source_cache,
                )
                .await?
                .is_none()
        {
            self.init_default_config_schema_store().await.1?;
        }
        Ok(())
    }

    pub async fn refresh_cache(&self) -> Result<bool, tombi_schema_store::Error> {
        let mut source_cache = self.config_source_cache.lock().await;
        if !self.backend_options.offline.unwrap_or_default() {
            source_cache.clear();
        }
        let mut updated = false;

        let config_paths: Vec<PathBuf> = self
            .config_schema_stores
            .read()
            .await
            .keys()
            .cloned()
            .collect();
        for config_path in config_paths {
            match source_cache
                .try_from_path(&config_path, self.http_client.as_ref())
                .await
            {
                Ok(Some(report)) => {
                    self.update_config_report(report, &config_path).await?;
                    updated = true;
                }
                Ok(None) => {}
                Err(err) => {
                    log::warn!(
                        "failed to reload config for {config_path}: {err}",
                        config_path = config_path.display()
                    );
                }
            }
        }

        // Keep `associate_schema` out while the associations are re-applied.
        let associated_schemas = self.associated_schemas.read().await;
        for ConfigSchemaStore {
            config,
            config_path,
            schema_store,
        } in self.all_config_schema_stores().await
        {
            if schema_store
                .refresh_cache(&config, config_path.as_deref())
                .await?
            {
                load_associations(&schema_store, &associated_schemas).await;
                updated = true;
            }
        }

        Ok(updated)
    }

    /// Update editor configuration
    pub async fn update_editor_config(&self, config: Config) {
        let (schema_store, applied, result) = self.build_schema_store(&config, None).await;
        if let Err(err) = result {
            log::error!("failed to load editor config schema store: {err}");
        }
        let config_schema_store = ConfigSchemaStore::new(config, None, schema_store);

        let _associations = self
            .catch_up_associations(&config_schema_store.schema_store, applied)
            .await;
        *self.default_config_schema_store.write().await =
            Some((DefaultConfigSource::Editor, config_schema_store));
    }

    /// Get editor configuration
    pub async fn default_config(&self) -> (DefaultConfigSource, Config) {
        let default_config_schema_store = self.default_config_schema_store.read().await;
        if let Some((source, config_schema_store)) = &*default_config_schema_store {
            (*source, config_schema_store.config.clone())
        } else {
            (DefaultConfigSource::Default, Config::default())
        }
    }

    /// List all schemas from all config schema stores
    pub async fn list_schemas(&self) -> Vec<tombi_schema_store::Schema> {
        let mut schema_map: tombi_hashmap::IndexMap<SchemaUri, tombi_schema_store::Schema> =
            tombi_hashmap::IndexMap::new();

        for config_schema_store in self.all_config_schema_stores().await {
            for schema in config_schema_store.schema_store.list_schemas().await {
                if let Some(existing_schema) = schema_map.get_mut(&schema.schema_uri) {
                    merge_schema(existing_schema, schema);
                } else {
                    schema_map.insert(schema.schema_uri.clone(), schema);
                }
            }
        }

        schema_map.into_values().collect()
    }
}

/// Load config and associated schemas into an existing SchemaStore
async fn load_schema_store_with_associations(
    schema_store: &SchemaStore,
    config: &Config,
    config_path: Option<&Path>,
    associated_schemas: &[AssociatedSchema],
) -> Result<(), tombi_schema_store::Error> {
    schema_store.load_config(config, config_path).await?;

    load_associations(schema_store, associated_schemas).await;

    Ok(())
}

async fn load_associations(schema_store: &SchemaStore, associated_schemas: &[AssociatedSchema]) {
    for associated_schema in associated_schemas {
        apply_association(schema_store, associated_schema).await;
    }
}

async fn apply_association(schema_store: &SchemaStore, associated_schema: &AssociatedSchema) {
    schema_store
        .associate_schema(
            associated_schema.schema_uri.clone(),
            associated_schema.file_match.clone(),
            &associated_schema.options,
        )
        .await;
}

fn schema_store_options(
    config: &Config,
    backend_options: &crate::backend::Options,
) -> tombi_schema_store::Options {
    tombi_schema_store::Options {
        offline: backend_options.offline,
        strict: config.schema.as_ref().and_then(|schema| schema.strict()),
        format: config
            .schema
            .as_ref()
            .and_then(|schema| schema.format.clone()),
        lint: config
            .schema
            .as_ref()
            .and_then(|schema| schema.lint.clone()),
        cache: Some(backend_options.cache.clone()),
        trusted_hosts: None,
    }
}

fn merge_schema(
    existing_schema: &mut tombi_schema_store::Schema,
    new_schema: tombi_schema_store::Schema,
) {
    if let (None, Some(title)) = (&existing_schema.title, new_schema.title) {
        existing_schema.title = Some(title);
    }
    if let (None, Some(description)) = (&existing_schema.description, new_schema.description) {
        existing_schema.description = Some(description);
    }
    for pattern in new_schema.include {
        if !existing_schema.include.contains(&pattern) {
            existing_schema.include.push(pattern);
        }
    }
    if let Some(new_exclude) = new_schema.exclude {
        let existing_exclude = existing_schema.exclude.get_or_insert_with(Vec::new);
        for pattern in new_exclude {
            if !existing_exclude.contains(&pattern) {
                existing_exclude.push(pattern);
            }
        }
    }
}

/// Configs built from the local config file `path`, itself included.
fn dependents_of(
    stores: &tombi_hashmap::HashMap<PathBuf, LoadedConfig>,
    path: &Path,
) -> Vec<PathBuf> {
    let selected = absolute_normalized_path(path);
    let identity = tombi_fs::canonicalize(path).unwrap_or_else(|_| selected.clone());
    let mut dependents = stores
        .iter()
        .filter(|(_, config)| {
            config
                .local_dependencies
                .iter()
                .any(|dependency| *dependency == selected || *dependency == identity)
        })
        .map(|(config_path, _)| config_path.clone())
        .collect::<Vec<_>>();
    dependents.sort();
    dependents
}

fn absolute_normalized_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_owned())
    };
    tombi_fs::normalize(&absolute)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(path: &Path, contents: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    async fn toml_version(
        manager: &ConfigManager,
        directory: &Path,
    ) -> Option<tombi_config::TomlVersion> {
        manager
            .config_schema_store_for_file(&directory.join("source.toml"))
            .await
            .config
            .toml_version
    }

    #[tokio::test]
    async fn reload_config_reloads_configs_extending_it() {
        let _cache_home = tombi_test_lib::TestCacheHome::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("shared/tombi.toml");
        let app_dir = temp_dir.path().join("app");
        write_file(&base_path, "toml-version = \"v1.0.0\"\n");
        write_file(
            &app_dir.join("tombi.toml"),
            "extends = [\"../shared/tombi.toml\"]\n",
        );

        let manager = ConfigManager::new(&crate::backend::Options {
            offline: Some(true),
            ..Default::default()
        });
        assert_eq!(
            toml_version(&manager, &app_dir).await,
            Some(tombi_config::TomlVersion::V1_0_0)
        );

        write_file(&base_path, "toml-version = \"v1.1.0\"\n");
        assert!(manager.reload_config(&base_path).await.unwrap());

        assert_eq!(
            toml_version(&manager, &app_dir).await,
            Some(tombi_config::TomlVersion::V1_1_0)
        );
    }

    #[tokio::test]
    async fn reload_config_keeps_previous_config_when_extends_is_invalid() {
        let _cache_home = tombi_test_lib::TestCacheHome::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("tombi.toml");
        write_file(&config_path, "toml-version = \"v1.0.0\"\n");

        let manager = ConfigManager::new(&crate::backend::Options {
            offline: Some(true),
            ..Default::default()
        });
        assert_eq!(
            toml_version(&manager, temp_dir.path()).await,
            Some(tombi_config::TomlVersion::V1_0_0)
        );

        for extends in [
            "missing/tombi.toml",
            "base.toml",
            "ftp://example.org/tombi.toml",
        ] {
            write_file(
                &config_path,
                &format!("toml-version = \"v1.1.0\"\nextends = \"{extends}\"\n"),
            );
            write_file(&temp_dir.path().join("base.toml"), "");

            assert!(!manager.reload_config(&config_path).await.unwrap());
            assert_eq!(
                toml_version(&manager, temp_dir.path()).await,
                Some(tombi_config::TomlVersion::V1_0_0),
                "extends = {extends}"
            );
        }
    }

    #[tokio::test]
    async fn reload_config_keeps_dependents_when_extended_config_becomes_invalid() {
        let _cache_home = tombi_test_lib::TestCacheHome::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path().join("shared/tombi.toml");
        let app_dir = temp_dir.path().join("app");
        write_file(&base_path, "toml-version = \"v1.0.0\"\n");
        write_file(
            &app_dir.join("tombi.toml"),
            "extends = \"../shared/tombi.toml\"\n",
        );

        let manager = ConfigManager::new(&crate::backend::Options {
            offline: Some(true),
            ..Default::default()
        });
        assert_eq!(
            toml_version(&manager, &app_dir).await,
            Some(tombi_config::TomlVersion::V1_0_0)
        );

        write_file(
            &base_path,
            "toml-version = \"v1.1.0\"\nextends = \"missing/tombi.toml\"\n",
        );
        assert!(!manager.reload_config(&base_path).await.unwrap());

        assert_eq!(
            toml_version(&manager, &app_dir).await,
            Some(tombi_config::TomlVersion::V1_0_0)
        );
    }

    #[test]
    fn dependents_of_forgets_removed_dependencies() {
        let config_path = PathBuf::from("/app/tombi.toml");
        let base_path = PathBuf::from("/shared/tombi.toml");
        let loaded = |local_dependencies: Vec<PathBuf>| LoadedConfig {
            store: ConfigSchemaStore::new(
                Config::default(),
                Some(config_path.clone()),
                SchemaStore::new(),
            ),
            local_dependencies,
        };
        let mut stores = tombi_hashmap::HashMap::new();

        stores.insert(
            config_path.clone(),
            loaded(vec![config_path.clone(), base_path.clone()]),
        );
        assert_eq!(
            dependents_of(&stores, &base_path),
            vec![config_path.clone()]
        );

        stores.insert(config_path.clone(), loaded(vec![config_path.clone()]));
        assert!(dependents_of(&stores, &base_path).is_empty());
    }
}
