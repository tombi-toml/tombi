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

/// Manages configuration files and their associated schema stores for different source files
#[derive(Debug)]
pub struct ConfigManager {
    /// Maps source file paths to their associated config file paths
    source_config_paths: Arc<tokio::sync::RwLock<tombi_hashmap::HashMap<PathBuf, PathBuf>>>,
    /// Maps config file paths to their ConfigSchemaStore
    config_schema_stores:
        Arc<tokio::sync::RwLock<tombi_hashmap::HashMap<PathBuf, ConfigSchemaStore>>>,
    config_dependency_graph: tokio::sync::RwLock<ConfigDependencyGraph>,
    config_source_cache: tokio::sync::Mutex<serde_tombi::config::ConfigSourceCache>,
    schema_store_operation_lock: tokio::sync::Mutex<()>,
    /// Default ConfigSchemaStore when no config file is found
    /// Contains the source type (Default or Editor) and the ConfigSchemaStore
    default_config_schema_store:
        Arc<tokio::sync::RwLock<Option<(DefaultConfigSource, ConfigSchemaStore)>>>,

    backend_options: crate::backend::Options,
    associated_schemas: Arc<tokio::sync::RwLock<Vec<AssociatedSchema>>>,
}

/// Which configs inherit from which local config files via `extends`.
#[derive(Debug, Default)]
struct ConfigDependencyGraph {
    /// config path -> local config files it is built from (itself included)
    dependencies: tombi_hashmap::HashMap<PathBuf, Vec<PathBuf>>,
    /// local config file -> config paths built from it (itself included)
    dependents: tombi_hashmap::HashMap<PathBuf, tombi_hashmap::HashSet<PathBuf>>,
}

impl ConfigDependencyGraph {
    fn set(&mut self, config_path: &Path, dependencies: Vec<PathBuf>) {
        self.remove(config_path);
        for dependency in &dependencies {
            self.dependents
                .entry(dependency.clone())
                .or_default()
                .insert(config_path.to_owned());
        }
        self.dependencies
            .insert(config_path.to_owned(), dependencies);
    }

    fn remove(&mut self, config_path: &Path) {
        for dependency in self.dependencies.remove(config_path).unwrap_or_default() {
            if let Some(dependents) = self.dependents.get_mut(&dependency) {
                dependents.remove(config_path);
                if dependents.is_empty() {
                    self.dependents.remove(&dependency);
                }
            }
        }
    }

    fn dependents_of(&self, path: &Path) -> Vec<PathBuf> {
        let selected = absolute_normalized_path(path);
        let identity = tombi_fs::canonicalize(path).unwrap_or_else(|_| selected.clone());
        let mut dependents = [selected, identity]
            .iter()
            .filter_map(|path| self.dependents.get(path))
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        dependents.sort();
        dependents.dedup();
        dependents
    }
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
            config_dependency_graph: tokio::sync::RwLock::new(ConfigDependencyGraph::default()),
            config_source_cache: tokio::sync::Mutex::new(Default::default()),
            schema_store_operation_lock: tokio::sync::Mutex::new(()),
            default_config_schema_store: Arc::new(tokio::sync::RwLock::new(None)),
            backend_options: backend_options.clone(),
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
        if let Some(store) = self.cached_store_for_file(text_document_path).await {
            return store;
        }
        let mut source_cache = self.config_source_cache.lock().await;
        if let Some(store) = self.cached_store_for_file(text_document_path).await {
            return store;
        }
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

    async fn cached_store_for_file(&self, path: &Path) -> Option<ConfigSchemaStore> {
        let config_path = self.source_config_paths.read().await.get(path).cloned()?;
        self.config_schema_stores
            .read()
            .await
            .get(&config_path)
            .cloned()
    }

    pub async fn config_schema_store_for_directory(
        &self,
        directory: Option<PathBuf>,
    ) -> ConfigSchemaStore {
        let mut source_cache = self.config_source_cache.lock().await;
        self.config_schema_store_for_directory_locked(directory, &mut source_cache)
            .await
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
            if let Some(store) = self.config_schema_stores.read().await.get(&candidate.path) {
                return Ok(Some(store.clone()));
            }
            if !tombi_fs::is_file(&candidate.path) {
                continue;
            }
            match source_cache
                .try_from_path(&candidate.path, self.load_options())
                .await
            {
                Ok(Some(report)) => {
                    let _operation = self.schema_store_operation_lock.lock().await;
                    self.update_config_report_locked(report, &candidate.path)
                        .await?;
                    return Ok(Some(
                        self.config_schema_stores
                            .read()
                            .await
                            .get(&candidate.path)
                            .expect("loaded config store exists")
                            .clone(),
                    ));
                }
                Ok(None) => {}
                Err(error @ tombi_config::Error::ConfigOffline { .. }) => {
                    log::warn!("failed to load config {:?}: {error}", candidate.path);
                    break;
                }
                Err(error) => log::warn!("failed to load config {:?}: {error}", candidate.path),
            }
        }
        Ok(None)
    }

    fn load_options(&self) -> tombi_config::LoadOptions {
        tombi_config::LoadOptions {
            offline: self.is_offline(),
        }
    }

    fn is_offline(&self) -> bool {
        self.backend_options.offline.unwrap_or(false)
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
            self.config_dependency_graph
                .read()
                .await
                .dependents_of(config_path)
                .into_iter()
                .filter(|dependent| dependent != config_path),
        );

        let mut reloaded = false;
        for config_path in config_paths {
            match source_cache
                .try_from_path(&config_path, self.load_options())
                .await
            {
                Ok(Some(report)) => {
                    let _operation = self.schema_store_operation_lock.lock().await;
                    self.update_config_report_locked(report, &config_path)
                        .await?;
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
        let _schema_store_operation = self.schema_store_operation_lock.lock().await;
        self.update_config_report_locked(
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
        .await
    }

    async fn update_config_report_locked(
        &self,
        report: serde_tombi::config::ConfigLoadReport,
        config_path: &Path,
    ) -> Result<(), tombi_schema_store::Error> {
        let serde_tombi::config::ConfigLoadReport {
            config,
            local_dependencies,
        } = report;
        let associated_schemas = self.associated_schemas.read().await.clone();
        let schema_options = schema_store_options(&config, &self.backend_options);
        let schema_store = SchemaStore::new_with_options(schema_options);
        load_schema_store_with_associations(
            &schema_store,
            &config,
            Some(config_path),
            &associated_schemas,
        )
        .await?;

        let mut config_schema_stores = self.config_schema_stores.write().await;
        config_schema_stores.insert(
            config_path.to_owned(),
            ConfigSchemaStore::new(config, Some(config_path.to_owned()), schema_store),
        );
        drop(config_schema_stores);
        self.config_dependency_graph
            .write()
            .await
            .set(config_path, local_dependencies);

        Ok(())
    }

    /// Get the default config
    async fn default_config_schema_store(&self) -> ConfigSchemaStore {
        if let Some((_, config_schema_store)) =
            self.default_config_schema_store.read().await.as_ref()
        {
            return config_schema_store.clone();
        }

        let _schema_store_operation = self.schema_store_operation_lock.lock().await;
        if let Some((_, config_schema_store)) =
            self.default_config_schema_store.read().await.as_ref()
        {
            return config_schema_store.clone();
        }

        let config = Config::default();
        let associated_schemas = self.associated_schemas.read().await.clone();
        let schema_options = schema_store_options(&config, &self.backend_options);
        let schema_store = SchemaStore::new_with_options(schema_options);

        if let Err(err) =
            load_schema_store_with_associations(&schema_store, &config, None, &associated_schemas)
                .await
        {
            log::error!("failed to load default schema store: {err}");
        }

        let config_schema_store = ConfigSchemaStore::new(config, None, schema_store);
        let mut default_config_schema_store = self.default_config_schema_store.write().await;
        if let Some((_, existing)) = default_config_schema_store.as_ref() {
            existing.clone()
        } else {
            *default_config_schema_store =
                Some((DefaultConfigSource::Default, config_schema_store.clone()));
            config_schema_store
        }
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
        let _schema_store_operation = self.schema_store_operation_lock.lock().await;
        let mut updated = false;
        let config_schema_stores = self
            .config_schema_stores
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for config_schema_store in config_schema_stores {
            updated |= config_schema_store
                .schema_store
                .update_schema(schema_uri.clone())
                .await?;
        }
        if let Some((_, ConfigSchemaStore { schema_store, .. })) =
            self.default_config_schema_store.read().await.clone()
        {
            updated |= schema_store.update_schema(schema_uri).await?;
        }
        Ok(updated)
    }

    pub async fn associate_schema(
        &self,
        schema_uri: &SchemaUri,
        file_match: &[String],
        options: &tombi_schema_store::AssociateSchemaOptions,
    ) {
        let _schema_store_operation = self.schema_store_operation_lock.lock().await;
        let mut associated_schemas = self.associated_schemas.write().await;
        let associated_schema = AssociatedSchema {
            schema_uri: schema_uri.clone(),
            file_match: file_match.to_vec(),
            options: options.clone(),
        };
        associated_schemas.push(associated_schema.clone());

        let config_schema_stores = {
            self.config_schema_stores
                .read()
                .await
                .values()
                .cloned()
                .collect::<Vec<_>>()
        };
        for config_schema_store in config_schema_stores {
            apply_association(&config_schema_store.schema_store, &associated_schema).await;
        }
        let default_config_schema_store = self.default_config_schema_store.read().await.clone();
        if let Some((_, ConfigSchemaStore { schema_store, .. })) = default_config_schema_store {
            apply_association(&schema_store, &associated_schema).await;
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
            && self.default_config_schema_store.read().await.is_none()
        {
            let _operation = self.schema_store_operation_lock.lock().await;
            if self.default_config_schema_store.read().await.is_none() {
                let config = Config::default();
                let associated_schemas = self.associated_schemas.read().await.clone();
                let schema_options = schema_store_options(&config, &self.backend_options);
                let schema_store = SchemaStore::new_with_options(schema_options);
                load_schema_store_with_associations(
                    &schema_store,
                    &config,
                    None,
                    &associated_schemas,
                )
                .await?;
                *self.default_config_schema_store.write().await = Some((
                    DefaultConfigSource::Default,
                    ConfigSchemaStore::new(config, None, schema_store),
                ));
            }
        }
        Ok(())
    }

    pub async fn refresh_cache(&self) -> Result<bool, tombi_schema_store::Error> {
        let mut source_cache = self.config_source_cache.lock().await;
        if !self.is_offline() {
            source_cache.clear();
        }
        let _schema_store_operation = self.schema_store_operation_lock.lock().await;
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
                .try_from_path(&config_path, self.load_options())
                .await
            {
                Ok(Some(report)) => {
                    self.update_config_report_locked(report, &config_path)
                        .await?;
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

        let config_schema_stores = self
            .config_schema_stores
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let default_config_schema_store = self.default_config_schema_store.read().await.clone();
        let associated_schemas = self.associated_schemas.read().await.clone();
        for ConfigSchemaStore {
            config,
            config_path,
            schema_store,
        } in config_schema_stores
        {
            if schema_store
                .refresh_cache(&config, config_path.as_deref())
                .await?
            {
                load_associations(&schema_store, &associated_schemas).await;
                updated = true;
            }
        }

        if let Some((
            _,
            ConfigSchemaStore {
                config,
                config_path,
                schema_store,
            },
        )) = default_config_schema_store
            && schema_store
                .refresh_cache(&config, config_path.as_deref())
                .await?
        {
            load_associations(&schema_store, &associated_schemas).await;
            updated = true;
        }

        Ok(updated)
    }

    /// Update editor configuration
    pub async fn update_editor_config(&self, config: Config) {
        let _schema_store_operation = self.schema_store_operation_lock.lock().await;
        let associated_schemas = self.associated_schemas.read().await.clone();
        let schema_options = schema_store_options(&config, &self.backend_options);
        let schema_store = SchemaStore::new_with_options(schema_options);

        if let Err(err) =
            load_schema_store_with_associations(&schema_store, &config, None, &associated_schemas)
                .await
        {
            log::error!("failed to load editor config schema store: {err}");
        }

        let config_schema_store = ConfigSchemaStore::new(config, None, schema_store);
        let mut default_config_schema_store = self.default_config_schema_store.write().await;
        *default_config_schema_store = Some((DefaultConfigSource::Editor, config_schema_store));
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

        // Get schemas from all config_schema_stores
        let config_schema_stores = self
            .config_schema_stores
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for config_schema_store in config_schema_stores {
            for schema in config_schema_store.schema_store.list_schemas().await {
                if let Some(existing_schema) = schema_map.get_mut(&schema.schema_uri) {
                    merge_schema(existing_schema, schema);
                } else {
                    schema_map.insert(schema.schema_uri.clone(), schema);
                }
            }
        }

        // Get schemas from default_config_schema_store
        if let Some((_, default_config_schema_store)) =
            self.default_config_schema_store.read().await.clone()
        {
            let schemas = default_config_schema_store
                .schema_store
                .list_schemas()
                .await;
            for schema in schemas {
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
            .config_schema_store_for_directory(Some(directory.to_owned()))
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
    fn dependency_graph_forgets_removed_dependencies() {
        let mut graph = ConfigDependencyGraph::default();
        let config_path = PathBuf::from("/app/tombi.toml");
        let base_path = PathBuf::from("/shared/tombi.toml");

        graph.set(&config_path, vec![config_path.clone(), base_path.clone()]);
        assert_eq!(graph.dependents_of(&base_path), vec![config_path.clone()]);

        graph.set(&config_path, vec![config_path.clone()]);
        assert!(graph.dependents_of(&base_path).is_empty());
    }
}
