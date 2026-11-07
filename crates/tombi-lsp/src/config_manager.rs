use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

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
    config_schema_versions: tokio::sync::RwLock<std::collections::HashMap<PathBuf, u64>>,
    config_update_sequence: AtomicU64,
    /// Default ConfigSchemaStore when no config file is found
    /// Contains the source type (Default or Editor) and the ConfigSchemaStore
    default_config_schema_store:
        Arc<tokio::sync::RwLock<Option<(DefaultConfigSource, ConfigSchemaStore)>>>,

    backend_options: crate::backend::Options,
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
            config_schema_versions: tokio::sync::RwLock::new(std::collections::HashMap::new()),
            config_update_sequence: AtomicU64::new(0),
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

    /// Get or compute the config for a given TOML file path
    pub async fn config_schema_store_for_file(
        &self,
        text_document_path: &Path,
    ) -> ConfigSchemaStore {
        if let Some(config_path) = self
            .source_config_paths
            .read()
            .await
            .get(text_document_path)
            .cloned()
            && let Some(config_schema_store) = self
                .config_schema_stores
                .read()
                .await
                .get(&config_path)
                .cloned()
        {
            return config_schema_store;
        }

        // Resolve inherited files before acquiring shared map locks so remote reads do not
        // keep config-map state locked while the network is in flight.
        let text_document_path_buf = text_document_path.to_path_buf();
        let Ok((config, Some(config_path))) = serde_tombi::config::load_with_path_async(
            text_document_path_buf.parent().map(ToOwned::to_owned),
        )
        .await
        else {
            return self.default_config_schema_store().await;
        };

        self.source_config_paths
            .write()
            .await
            .insert(text_document_path_buf, config_path.clone());

        if let Some(config_schema_store) = self
            .config_schema_stores
            .read()
            .await
            .get(&config_path)
            .cloned()
        {
            return config_schema_store;
        }

        if let Err(error) = self.update_config_with_path(config, &config_path).await {
            log::error!("failed to load schema store for {config_path:?}: {error}");
            return self.default_config_schema_store().await;
        }
        self.config_schema_stores
            .read()
            .await
            .get(&config_path)
            .cloned()
            .unwrap_or_else(|| ConfigSchemaStore::new(Config::default(), None, SchemaStore::new()))
    }

    /// Update a specific config and its path
    pub async fn update_config_with_path(
        &self,
        config: Config,
        config_path: &Path,
    ) -> Result<(), tombi_schema_store::Error> {
        let sequence = self.config_update_sequence.fetch_add(1, Ordering::Relaxed);
        // `SchemaStore` options are fixed at construction, so a config change builds a new store.
        let associated_schemas = self.associated_schemas.read().await.clone();
        let associated_schema_count = associated_schemas.len();
        let schema_options = schema_store_options(&config, &self.backend_options);
        let schema_store = SchemaStore::new_with_options(schema_options);
        load_schema_store_with_associations(
            &schema_store,
            &config,
            Some(config_path),
            &associated_schemas,
        )
        .await?;

        // Load the replacement before taking the shared map lock. If concurrent reloads finish
        // out of order, the per-path sequence keeps the newer config in the map. The association
        // lock closes the gap between the snapshot above and publishing the replacement.
        let _associated_schemas = self
            .synchronize_associations(&schema_store, associated_schema_count)
            .await;
        let mut config_schema_versions = self.config_schema_versions.write().await;
        if config_schema_versions
            .get(config_path)
            .is_some_and(|current| *current > sequence)
        {
            return Ok(());
        }
        let mut config_schema_stores = self.config_schema_stores.write().await;
        config_schema_versions.insert(config_path.to_owned(), sequence);
        config_schema_stores.insert(
            config_path.to_owned(),
            ConfigSchemaStore::new(config, Some(config_path.to_owned()), schema_store),
        );
        Ok(())
    }

    /// Get the default config
    async fn default_config_schema_store(&self) -> ConfigSchemaStore {
        if let Some((_, config_schema_store)) =
            self.default_config_schema_store.read().await.as_ref()
        {
            return config_schema_store.clone();
        }

        let (config, config_path) =
            match serde_tombi::config::load_with_path_async(std::env::current_dir().ok()).await {
                Ok((config, config_path)) => (config, config_path),
                Err(error) => {
                    log::warn!("failed to load default config: {error}");
                    (Config::default(), None)
                }
            };
        let associated_schemas = self.associated_schemas.read().await.clone();
        let associated_schema_count = associated_schemas.len();
        let schema_options = schema_store_options(&config, &self.backend_options);
        let schema_store = SchemaStore::new_with_options(schema_options);

        if let Err(err) = load_schema_store_with_associations(
            &schema_store,
            &config,
            config_path.as_deref(),
            &associated_schemas,
        )
        .await
        {
            log::error!("failed to load default schema store: {err}");
        }

        let config_schema_store = ConfigSchemaStore::new(config, config_path, schema_store);
        let _associated_schemas = self
            .synchronize_associations(&config_schema_store.schema_store, associated_schema_count)
            .await;
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
        // Serialize registration with store publication. The map locks are held only while
        // cloning stores; schema-store work runs after those guards are dropped.
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
        if self.config_schema_stores.read().await.is_empty()
            && self.default_config_schema_store.read().await.is_none()
        {
            match serde_tombi::config::load_with_path_and_level_async(std::env::current_dir().ok())
                .await
            {
                Ok((config, Some(config_path), _)) => {
                    self.update_config_with_path(config, &config_path).await?;
                }
                Ok((config, None, _)) => {
                    let associated_schemas = self.associated_schemas.read().await.clone();
                    let associated_schema_count = associated_schemas.len();
                    let schema_options = schema_store_options(&config, &self.backend_options);
                    let schema_store = SchemaStore::new_with_options(schema_options);
                    load_schema_store_with_associations(
                        &schema_store,
                        &config,
                        None,
                        &associated_schemas,
                    )
                    .await?;
                    let config_schema_store = ConfigSchemaStore::new(config, None, schema_store);
                    let _associated_schemas = self
                        .synchronize_associations(
                            &config_schema_store.schema_store,
                            associated_schema_count,
                        )
                        .await;
                    let mut default_config_schema_store =
                        self.default_config_schema_store.write().await;
                    if default_config_schema_store.is_none() {
                        *default_config_schema_store =
                            Some((DefaultConfigSource::Default, config_schema_store));
                    }
                }
                Err(error) => log::warn!("failed to discover startup config: {error}"),
            }
        }

        Ok(())
    }

    pub async fn refresh_cache(&self) -> Result<bool, tombi_schema_store::Error> {
        let mut updated = false;

        // Re-read config files from disk so that changes made outside the editor are applied.
        // Collect the keys so the read lock is released before `update_config_with_path` takes the write lock.
        let config_paths: Vec<PathBuf> = self
            .config_schema_stores
            .read()
            .await
            .keys()
            .cloned()
            .collect();
        for config_path in config_paths {
            match serde_tombi::config::try_from_path_async(&config_path).await {
                Ok(Some(config)) => {
                    self.update_config_with_path(config, &config_path).await?;
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
        let associated_schemas = self.associated_schemas.read().await.clone();
        let associated_schema_count = associated_schemas.len();
        let schema_options = schema_store_options(&config, &self.backend_options);
        let schema_store = SchemaStore::new_with_options(schema_options);

        if let Err(err) =
            load_schema_store_with_associations(&schema_store, &config, None, &associated_schemas)
                .await
        {
            log::error!("failed to load editor config schema store: {err}");
        }

        let config_schema_store = ConfigSchemaStore::new(config, None, schema_store);
        let _associated_schemas = self
            .synchronize_associations(&config_schema_store.schema_store, associated_schema_count)
            .await;
        let mut default_config_schema_store = self.default_config_schema_store.write().await;
        *default_config_schema_store = Some((DefaultConfigSource::Editor, config_schema_store));
    }

    /// Catch up a store created from an earlier association snapshot and hold the association
    /// lock until the caller publishes it. `associate_schema` uses the same lock through its
    /// updates to currently published stores, so each registration is applied exactly once.
    async fn synchronize_associations(
        &self,
        schema_store: &SchemaStore,
        applied_association_count: usize,
    ) -> tokio::sync::RwLockWriteGuard<'_, Vec<AssociatedSchema>> {
        let associated_schemas = self.associated_schemas.write().await;
        load_associations(
            schema_store,
            &associated_schemas[applied_association_count..],
        )
        .await;
        associated_schemas
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
