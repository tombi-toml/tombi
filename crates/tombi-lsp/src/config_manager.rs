use std::path::{Path, PathBuf};
use std::sync::Arc;

use ahash::AHashMap;
use indexmap::IndexMap;
use tombi_config::{Config, TomlVersion};
use tombi_schema_store::{SchemaStore, SchemaUri};
use tower_lsp::lsp_types::Url;

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
    source_config_paths: Arc<tokio::sync::RwLock<AHashMap<PathBuf, PathBuf>>>,
    /// Maps config file paths to their ConfigSchemaStore
    config_schema_stores: Arc<tokio::sync::RwLock<AHashMap<PathBuf, ConfigSchemaStore>>>,
    /// Default ConfigSchemaStore when no config file is found
    /// Contains the source type (Default or Editor) and the ConfigSchemaStore
    default_config_schema_store:
        Arc<tokio::sync::RwLock<Option<(DefaultConfigSource, ConfigSchemaStore)>>>,

    backend_options: crate::backend::Options,
    associated_schemas: Arc<tokio::sync::RwLock<Vec<AssociatedSchema>>>,
}

#[derive(Debug)]
struct AssociatedSchema {
    schema_uri: SchemaUri,
    file_match: Vec<String>,
    toml_version: Option<TomlVersion>,
}

impl ConfigManager {
    pub fn new(backend_options: &crate::backend::Options) -> Self {
        // Try to load the default config
        let (config, config_path) =
            match serde_tombi::config::load_with_path(std::env::current_dir().ok()) {
                Ok((config, config_path)) => (config, config_path),
                Err(err) => {
                    tracing::error!("Failed to load default config: {err}");
                    (Config::default(), None)
                }
            };

        // Initialize config_schema_stores with the default config if it has a path
        let mut config_schema_stores = AHashMap::new();
        let mut default_config_schema_store = None;
        if let Some(config_path) = config_path {
            let schema_options = schema_store_options(&config, backend_options);
            config_schema_stores.insert(
                config_path.clone(),
                ConfigSchemaStore::new(
                    config,
                    Some(config_path),
                    SchemaStore::new_with_options(schema_options),
                ),
            );
        } else {
            let schema_options = schema_store_options(&config, backend_options);
            default_config_schema_store = Some((
                DefaultConfigSource::Default,
                ConfigSchemaStore::new(config, None, SchemaStore::new_with_options(schema_options)),
            ));
        }

        Self {
            source_config_paths: Arc::new(tokio::sync::RwLock::new(AHashMap::new())),
            config_schema_stores: Arc::new(tokio::sync::RwLock::new(config_schema_stores)),
            default_config_schema_store: Arc::new(tokio::sync::RwLock::new(
                default_config_schema_store,
            )),
            backend_options: backend_options.clone(),
            associated_schemas: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
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
        // Check if we already have a config path for this source file
        let mut source_config_paths = self.source_config_paths.write().await;
        let config_path: PathBuf = match source_config_paths.get(text_document_path) {
            Some(config_path) => config_path.to_owned(),
            None => {
                let text_document_path_buf: PathBuf = text_document_path.to_path_buf();
                if let Ok((config, Some(config_path_buf))) = serde_tombi::config::load_with_path(
                    text_document_path_buf.parent().map(ToOwned::to_owned),
                ) {
                    source_config_paths.insert(text_document_path_buf, config_path_buf.clone());

                    let schema_options = schema_store_options(&config, &self.backend_options);
                    let mut config_schema_stores = self.config_schema_stores.write().await;
                    let ConfigSchemaStore {
                        config,
                        schema_store,
                        ..
                    } = config_schema_stores
                        .entry(config_path_buf.clone())
                        .or_insert_with(|| {
                            ConfigSchemaStore::new(
                                config,
                                Some(config_path_buf.clone()),
                                SchemaStore::new_with_options(schema_options),
                            )
                        });

                    if schema_store.is_empty().await {
                        tracing::info!("Add new SchemaStore for {config_path_buf:?}");
                        let associated_schemas = self.associated_schemas.read().await;
                        if let Err(err) = load_schema_store_with_associations(
                            schema_store,
                            config,
                            Some(&config_path_buf),
                            &associated_schemas,
                        )
                        .await
                        {
                            tracing::error!("Failed to load schema store: {err}");
                        }
                    }

                    config_path_buf
                } else {
                    return self.default_config_schema_store().await;
                }
            }
        };

        let config_schema_stores = self.config_schema_stores.read().await;
        if let Some(config_schema_store) = config_schema_stores.get(&config_path) {
            config_schema_store.clone()
        } else {
            self.default_config_schema_store().await
        }
    }

    /// Update a specific config and its path
    pub async fn update_config_with_path(
        &self,
        config: Config,
        config_path: &Path,
    ) -> Result<(), tombi_schema_store::Error> {
        let schema_options = schema_store_options(&config, &self.backend_options);

        let mut config_schema_stores = self.config_schema_stores.write().await;
        let config_schema_store = config_schema_stores
            .entry(config_path.to_owned())
            .or_insert(ConfigSchemaStore::new(
                config.clone(),
                Some(config_path.to_owned()),
                SchemaStore::new_with_options(schema_options),
            ));
        config_schema_store
            .schema_store
            .reload_config(&config, Some(config_path))
            .await?;
        config_schema_store.config = config;
        Ok(())
    }

    /// Get the default config
    async fn default_config_schema_store(&self) -> ConfigSchemaStore {
        let mut default_config_schema_store = self.default_config_schema_store.write().await;
        if let Some((_, ref config_schema_store)) = *default_config_schema_store {
            config_schema_store.clone()
        } else {
            let config = Config::default();
            let associated_schemas = self.associated_schemas.read().await;
            let schema_options = schema_store_options(&config, &self.backend_options);
            let schema_store = SchemaStore::new_with_options(schema_options);

            if let Err(err) = load_schema_store_with_associations(
                &schema_store,
                &config,
                None,
                &associated_schemas,
            )
            .await
            {
                tracing::error!("Failed to load default schema store: {err}");
            }

            let config_schema_store = ConfigSchemaStore::new(config, None, schema_store);
            *default_config_schema_store =
                Some((DefaultConfigSource::Default, config_schema_store.clone()));
            config_schema_store
        }
    }

    /// Get the config path for a specific URL
    pub async fn get_config_path_for_url(&self, url: &Url) -> Option<PathBuf> {
        if let Ok(path) = url.to_file_path() {
            let source_config_paths = self.source_config_paths.read().await;
            if let Some(config_path) = source_config_paths.get(&path) {
                return Some(config_path.clone());
            }
        }
        None
    }

    pub async fn update_schema(
        &self,
        schema_uri: &SchemaUri,
    ) -> Result<bool, tombi_schema_store::Error> {
        let mut updated = false;
        let mut config_schema_stores = self.config_schema_stores.write().await;
        for config_schema_store in config_schema_stores.values_mut() {
            updated |= config_schema_store
                .schema_store
                .update_schema(schema_uri)
                .await?;
        }
        if let Some((_, ConfigSchemaStore { schema_store, .. })) =
            &mut *self.default_config_schema_store.write().await
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
        // Add to associated_schemas
        self.associated_schemas
            .write()
            .await
            .push(AssociatedSchema {
                schema_uri: schema_uri.clone(),
                file_match: file_match.to_vec(),
                toml_version: options.toml_version,
            });

        // Update config_schema_stores
        {
            let mut config_schema_stores = self.config_schema_stores.write().await;
            for config_schema_store in config_schema_stores.values_mut() {
                config_schema_store
                    .schema_store
                    .associate_schema(schema_uri.clone(), file_match.to_vec(), options)
                    .await;
            }
            if let Some((_, ConfigSchemaStore { schema_store, .. })) =
                &mut *self.default_config_schema_store.write().await
            {
                schema_store
                    .associate_schema(schema_uri.clone(), file_match.to_vec(), options)
                    .await;
            }
        }
    }

    pub async fn load(&self) -> Result<(), tombi_schema_store::Error> {
        let mut config_schema_stores = self.config_schema_stores.write().await;
        for (
            config_path,
            ConfigSchemaStore {
                config,
                schema_store,
                ..
            },
        ) in config_schema_stores.iter_mut()
        {
            schema_store.load_config(config, Some(config_path)).await?;
        }

        if let Some((
            _,
            ConfigSchemaStore {
                config,
                schema_store,
                ..
            },
        )) = &mut *self.default_config_schema_store.write().await
        {
            schema_store.load_config(config, None).await?;
        }

        Ok(())
    }

    pub async fn refresh_cache(&self) -> Result<bool, tombi_schema_store::Error> {
        let mut updated = false;
        let mut config_schema_stores = self.config_schema_stores.write().await;
        for (
            config_path,
            ConfigSchemaStore {
                config,
                schema_store,
                ..
            },
        ) in config_schema_stores.iter_mut()
        {
            updated |= schema_store
                .refresh_cache(config, Some(config_path))
                .await?;
        }

        if let Some((
            _,
            ConfigSchemaStore {
                config,
                schema_store,
                ..
            },
        )) = &mut *self.default_config_schema_store.write().await
        {
            updated |= schema_store.refresh_cache(config, None).await?;
        }

        Ok(updated)
    }

    pub async fn load_config_schemas(
        &self,
        schemas: &[tombi_config::SchemaItem],
        base_dir_path: Option<&std::path::Path>,
    ) {
        let mut config_schema_stores = self.config_schema_stores.write().await;
        for ConfigSchemaStore { schema_store, .. } in config_schema_stores.values_mut() {
            schema_store
                .load_config_schemas(schemas, base_dir_path)
                .await;
        }

        if let Some((_, ConfigSchemaStore { schema_store, .. })) =
            &mut *self.default_config_schema_store.write().await
        {
            schema_store
                .load_config_schemas(schemas, base_dir_path)
                .await;
        }
    }

    /// Update editor configuration
    pub async fn update_editor_config(&self, config: Config) {
        let associated_schemas = self.associated_schemas.read().await;
        let schema_options = schema_store_options(&config, &self.backend_options);
        let schema_store = SchemaStore::new_with_options(schema_options);

        if let Err(err) =
            load_schema_store_with_associations(&schema_store, &config, None, &associated_schemas)
                .await
        {
            tracing::error!("Failed to load editor config schema store: {err}");
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
        let mut schema_map: IndexMap<SchemaUri, tombi_schema_store::Schema> = IndexMap::new();

        // Get schemas from all config_schema_stores
        let config_schema_stores = self.config_schema_stores.read().await;
        for config_schema_store in config_schema_stores.values() {
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
            &*self.default_config_schema_store.read().await
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

    for associated_schema in associated_schemas {
        schema_store
            .associate_schema(
                associated_schema.schema_uri.clone(),
                associated_schema.file_match.clone(),
                &tombi_schema_store::AssociateSchemaOptions {
                    toml_version: associated_schema.toml_version,
                    force: false,
                    title: None,
                    description: None,
                },
            )
            .await;
    }

    Ok(())
}

fn schema_store_options(
    config: &Config,
    backend_options: &crate::backend::Options,
) -> tombi_schema_store::Options {
    tombi_schema_store::Options {
        offline: backend_options.offline,
        strict: config.schema.as_ref().and_then(|schema| schema.strict()),
        cache: Some(tombi_cache::Options {
            no_cache: backend_options.no_cache,
            ..Default::default()
        }),
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
}
