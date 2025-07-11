use std::{ops::Deref, str::FromStr, sync::Arc};

use crate::{
    get_tombi_scheme_content, json::JsonCatalog, CatalogUrl, DocumentSchema, HttpClient,
    SchemaAccessor, SchemaAccessors, SchemaUrl, SourceSchema,
};
use ahash::AHashMap;
use itertools::Either;
use tokio::sync::RwLock;
use tombi_cache::{get_cache_file_path, read_from_cache, refresh_cache, save_to_cache};
use tombi_config::{Schema, SchemaOptions};
use tombi_future::{BoxFuture, Boxable};
use tombi_url::url_to_file_path;

#[derive(Debug, Clone)]
pub struct SchemaStore {
    http_client: HttpClient,
    document_schemas:
        Arc<tokio::sync::RwLock<AHashMap<SchemaUrl, Result<DocumentSchema, crate::Error>>>>,
    schemas: Arc<RwLock<Vec<crate::Schema>>>,
    options: crate::Options,
}

impl Default for SchemaStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaStore {
    /// New with default options
    ///
    /// Create an empty store.
    /// Note that the new() does not automatically load schemas from Config etc.
    pub fn new() -> Self {
        Self::new_with_options(crate::Options::default())
    }

    /// New with options
    ///
    /// Create a store with the given options.
    /// Note that the new_with_options() does not automatically load schemas from Config etc.
    pub fn new_with_options(options: crate::Options) -> Self {
        Self {
            http_client: HttpClient::new(),
            document_schemas: Arc::new(RwLock::default()),
            schemas: Arc::new(RwLock::new(Vec::new())),
            options,
        }
    }

    /// Offline mode
    fn offline(&self) -> bool {
        self.options.offline.unwrap_or(false)
    }

    /// Strict mode
    pub fn strict(&self) -> bool {
        self.options.strict.unwrap_or(true)
    }

    pub async fn refresh_cache(
        &self,
        config: &tombi_config::Config,
        config_path: Option<&std::path::Path>,
    ) -> Result<bool, crate::Error> {
        if refresh_cache().await? {
            self.reload_config(config, config_path).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn reload_config(
        &self,
        config: &tombi_config::Config,
        config_path: Option<&std::path::Path>,
    ) -> Result<(), crate::Error> {
        self.document_schemas.write().await.clear();
        self.schemas.write().await.clear();
        self.load_config(config, config_path).await?;
        Ok(())
    }

    pub async fn load_config(
        &self,
        config: &tombi_config::Config,
        config_path: Option<&std::path::Path>,
    ) -> Result<(), crate::Error> {
        let base_dir_path = config_path.and_then(|p| p.parent());
        let schema_options = match &config.schema {
            Some(schema) => schema,
            None => &SchemaOptions::default(),
        };

        if schema_options.enabled.unwrap_or_default().value() {
            self.load_schemas(
                match &config.schemas {
                    Some(schemas) => schemas,
                    None => &[],
                },
                base_dir_path,
            )
            .await;

            let catalog_paths = schema_options.catalog_paths().unwrap_or_default();

            let catalogs_results =
                futures::future::join_all(catalog_paths.iter().map(|catalog_path| async move {
                    let Ok(catalog_url) = catalog_path.try_to_catalog_url(base_dir_path) else {
                        return Err(crate::Error::CatalogPathConvertUrlFailed {
                            catalog_path: catalog_path.to_string(),
                        });
                    };
                    let catalog_url = CatalogUrl::new(catalog_url);
                    self.load_json_catalog_from_catalog_url(&catalog_url).await
                }))
                .await;

            for catalog_result in catalogs_results {
                match catalog_result {
                    Ok(Some(catalog)) => {
                        self.add_json_catalog(catalog).await?;
                    }
                    Ok(None) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(())
    }

    pub async fn load_schemas(&self, schemas: &[Schema], base_dir_path: Option<&std::path::Path>) {
        futures::future::join_all(schemas.iter().map(|schema| async move {
            let schema_url = if let Ok(schema_url) = SchemaUrl::parse(schema.path()) {
                schema_url
            } else if let Ok(schema_url) = match base_dir_path {
                Some(base_dir_path) => SchemaUrl::from_file_path(base_dir_path.join(schema.path())),
                None => SchemaUrl::from_file_path(schema.path()),
            } {
                schema_url
            } else {
                tracing::error!("invalid schema path: {}", schema.path());
                return;
            };

            tracing::debug!("load config schema from: {}", schema_url);

            self.schemas.write().await.push(crate::Schema {
                url: schema_url,
                include: schema.include().to_vec(),
                toml_version: schema.toml_version(),
                sub_root_keys: schema.root().and_then(SchemaAccessor::parse),
            });
        }))
        .await;
    }

    async fn load_json_catalog_from_catalog_url(
        &self,
        catalog_url: &CatalogUrl,
    ) -> Result<Option<JsonCatalog>, crate::Error> {
        Ok(Some(match catalog_url.scheme() {
            "http" | "https" => {
                let catalog_cache_path = get_cache_file_path(catalog_url).await;
                if let Some(catalog_cache_content) =
                    read_from_cache(catalog_cache_path.as_deref(), self.options.cache.as_ref())
                        .await?
                {
                    let catalog = serde_json::from_str(&catalog_cache_content).map_err(|err| {
                        crate::Error::InvalidJsonFormat {
                            url: catalog_url.deref().clone(),
                            reason: err.to_string(),
                        }
                    })?;
                    tracing::debug!("load catalog from cache: {}", catalog_url);
                    return Ok(Some(catalog));
                }
                if self.offline() {
                    tracing::debug!("offline mode, skip fetch catalog from url: {}", catalog_url);
                    return Ok(None);
                }
                tracing::debug!("loading schema catalog: {}", catalog_url);

                match self.http_client.get_bytes(catalog_url.as_str()).await {
                    Ok(bytes) => {
                        if let Err(err) = save_to_cache(catalog_cache_path.as_deref(), &bytes).await
                        {
                            tracing::error!("{err}");
                        }

                        match serde_json::from_slice::<crate::json::JsonCatalog>(&bytes) {
                            Ok(catalog) => catalog,
                            Err(err) => {
                                return Err(crate::Error::InvalidJsonFormat {
                                    url: catalog_url.deref().clone(),
                                    reason: err.to_string(),
                                })
                            }
                        }
                    }
                    Err(err) => {
                        return Err(crate::Error::CatalogUrlFetchFailed {
                            catalog_url: catalog_url.clone(),
                            reason: err.to_string(),
                        });
                    }
                }
            }
            "file" => {
                let catalog_path = url_to_file_path(catalog_url).map_err(|_| {
                    crate::Error::InvalidCatalogFileUrl {
                        catalog_url: catalog_url.clone(),
                    }
                })?;

                let content = std::fs::read_to_string(&catalog_path).map_err(|_| {
                    crate::Error::CatalogFileReadFailed {
                        catalog_path: catalog_path.to_path_buf(),
                    }
                })?;

                tracing::debug!("load catalog from file: {}", catalog_url);

                serde_json::from_str(&content).map_err(|err| crate::Error::InvalidJsonFormat {
                    url: catalog_url.deref().clone(),
                    reason: err.to_string(),
                })?
            }
            "tombi" => {
                if catalog_url.path() != "/json/catalog.json" {
                    return Err(crate::Error::InvalidCatalogFileUrl {
                        catalog_url: catalog_url.clone(),
                    });
                }

                tracing::debug!("load catalog from embedded file: {}", catalog_url);

                serde_json::from_str::<crate::json::JsonCatalog>(include_str!(
                    "../../../schemas/catalog.json"
                ))
                .map_err(|err| crate::Error::InvalidJsonFormat {
                    url: catalog_url.deref().clone(),
                    reason: err.to_string(),
                })?
            }
            _ => {
                return Err(crate::Error::UnsupportedUrlScheme {
                    url: catalog_url.deref().clone(),
                });
            }
        }))
    }

    async fn add_json_catalog(&self, json_catalog: JsonCatalog) -> Result<(), crate::Error> {
        let mut schemas = self.schemas.write().await;
        for schema in json_catalog.schemas {
            if schema
                .file_match
                .iter()
                .any(|pattern| pattern.ends_with(".toml"))
            {
                schemas.push(crate::Schema {
                    url: schema.url,
                    include: schema.file_match,
                    toml_version: None,
                    sub_root_keys: None,
                });
            }
        }
        Ok(())
    }

    pub async fn update_schema(&self, schema_url: &SchemaUrl) -> Result<bool, crate::Error> {
        if matches!(schema_url.scheme(), "http" | "https") && self.offline() {
            tracing::debug!("offline mode, skip fetch schema from url: {}", schema_url);
            return Ok(false);
        }

        let has_key = { self.document_schemas.read().await.contains_key(schema_url) };
        if has_key {
            if let Some(document_schema) = self.fetch_document_schema(schema_url).await.transpose()
            {
                self.document_schemas
                    .write()
                    .await
                    .insert(schema_url.clone(), document_schema);
                tracing::debug!("update schema: {}", schema_url);
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn fetch_schema_value(
        &self,
        schema_url: &SchemaUrl,
    ) -> Result<Option<tombi_json::ValueNode>, crate::Error> {
        match schema_url.scheme() {
            "file" => {
                let schema_path =
                    url_to_file_path(schema_url).map_err(|_| crate::Error::InvalidSchemaUrl {
                        schema_url: schema_url.to_string(),
                    })?;
                if !schema_path.exists() {
                    return Err(crate::Error::SchemaFileNotFound {
                        schema_path: schema_path.clone(),
                    });
                }

                let file = std::fs::File::open(&schema_path)
                    .map_err(|_| crate::Error::SchemaFileReadFailed { schema_path })?;

                tracing::debug!("fetch schema from file: {}", schema_url);

                Ok(Some(tombi_json::ValueNode::from_reader(file).map_err(
                    |err| crate::Error::SchemaFileParseFailed {
                        schema_url: schema_url.to_owned(),
                        reason: err.to_string(),
                    },
                )?))
            }
            "http" | "https" => {
                let schema_cache_path = get_cache_file_path(schema_url).await;
                if let Some(schema_cache_content) =
                    read_from_cache(schema_cache_path.as_deref(), self.options.cache.as_ref())
                        .await?
                {
                    tracing::debug!("fetch schema from cache: {}", schema_url);

                    return Ok(Some(
                        tombi_json::ValueNode::from_str(&schema_cache_content).map_err(|err| {
                            crate::Error::SchemaFileParseFailed {
                                schema_url: schema_url.to_owned(),
                                reason: err.to_string(),
                            }
                        })?,
                    ));
                }
                if self.offline() {
                    tracing::debug!("offline mode, skip fetch catalog from url: {}", schema_url);
                    return Ok(None);
                }

                tracing::debug!("fetch schema from url: {}", schema_url);

                let bytes = self
                    .http_client
                    .get_bytes(schema_url.as_ref())
                    .await
                    .map_err(|err| crate::Error::SchemaFetchFailed {
                        schema_url: schema_url.clone(),
                        reason: err.to_string(),
                    })?;

                if let Err(err) = save_to_cache(schema_cache_path.as_deref(), &bytes).await {
                    tracing::error!("{err}");
                }

                Ok(Some(
                    tombi_json::ValueNode::from_reader(std::io::Cursor::new(bytes)).map_err(
                        |err| crate::Error::SchemaFileParseFailed {
                            schema_url: schema_url.to_owned(),
                            reason: err.to_string(),
                        },
                    )?,
                ))
            }
            "tombi" => {
                let Some(content) = get_tombi_scheme_content(schema_url) else {
                    return Err(crate::Error::SchemaResourceNotFound {
                        schema_url: schema_url.to_owned(),
                    });
                };

                tracing::debug!("fetch schema from embedded file: {}", schema_url);

                Ok(Some(tombi_json::ValueNode::from_str(content).map_err(
                    |err| crate::Error::SchemaFileParseFailed {
                        schema_url: schema_url.to_owned(),
                        reason: err.to_string(),
                    },
                )?))
            }
            _ => Err(crate::Error::UnsupportedUrlScheme {
                url: schema_url.deref().clone(),
            }),
        }
    }

    async fn fetch_document_schema(
        &self,
        schema_url: &SchemaUrl,
    ) -> Result<Option<DocumentSchema>, crate::Error> {
        let object = match self.fetch_schema_value(schema_url).await? {
            Some(tombi_json::ValueNode::Object(object)) => object,
            Some(_) => {
                return Err(crate::Error::SchemaMustBeObject {
                    schema_url: schema_url.to_owned(),
                });
            }
            None => return Ok(None),
        };

        Ok(Some(DocumentSchema::new(object, schema_url.clone())))
    }

    pub fn try_get_document_schema<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
    ) -> BoxFuture<'b, Result<Option<DocumentSchema>, crate::Error>> {
        async move {
            if let Some(document_schema) = self.document_schemas.read().await.get(schema_url) {
                return match document_schema {
                    Ok(document_schema) => Ok(Some(document_schema.clone())),
                    Err(err) => Err(err.to_owned()),
                };
            }

            match self.fetch_document_schema(schema_url).await.transpose() {
                Some(document_schema) => {
                    self.document_schemas
                        .write()
                        .await
                        .insert(schema_url.clone(), document_schema.clone());

                    Some(document_schema).transpose()
                }
                None => Ok(None),
            }
        }
        .boxed()
    }

    #[inline]
    async fn try_get_source_schema_from_remote_url(
        &self,
        schema_url: &SchemaUrl,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        Ok(Some(SourceSchema {
            root_schema: self.try_get_document_schema(schema_url).await?,
            sub_schema_url_map: Default::default(),
        }))
    }

    pub async fn resolve_source_schema_from_ast(
        &self,
        root: &tombi_ast::Root,
        source_url_or_path: Option<Either<&url::Url, &std::path::Path>>,
    ) -> Result<Option<SourceSchema>, (crate::Error, tombi_text::Range)> {
        let source_path = match source_url_or_path {
            Some(Either::Left(url)) => match url.scheme() {
                "file" => url_to_file_path(url).ok(),
                _ => None,
            },
            Some(Either::Right(path)) => Some(path.to_path_buf()),
            None => None,
        };

        if let Some((schema_url, url_range)) = root.file_schema_url(source_path.as_deref()) {
            let schema_url = match schema_url {
                Ok(schema_url) => schema_url,
                Err(schema_url_or_file_path) => {
                    return Err((
                        crate::Error::InvalidSchemaUrlOrFilePath {
                            schema_url_or_file_path,
                        },
                        url_range,
                    ));
                }
            };
            return self
                .try_get_source_schema_from_remote_url(&SchemaUrl::new(schema_url))
                .await
                .map_err(|err| (err, url_range));
        }

        if let Some(source_url_or_path) = source_url_or_path {
            Ok(self
                .resolve_source_schema(source_url_or_path)
                .await
                .ok()
                .flatten())
        } else {
            Ok(None)
        }
    }

    async fn resolve_source_schema_from_path(
        &self,
        source_path: &std::path::Path,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        let schemas = self.schemas.read().await;
        let matching_schemas = schemas
            .iter()
            .filter(|schema| {
                schema.include.iter().any(|pat| {
                    let pattern = if !pat.contains("*") {
                        format!("**/{}", pat)
                    } else {
                        pat.to_string()
                    };
                    glob::Pattern::new(&pattern)
                        .ok()
                        .map(|glob_pat| glob_pat.matches_path(source_path))
                        .unwrap_or(false)
                })
            })
            .collect::<Vec<_>>();

        let mut source_schema: Option<SourceSchema> = None;
        for matching_schema in matching_schemas {
            // Skip if the same schema (by URL and sub_root_keys) is already loaded in source_schema
            let already_loaded = match &matching_schema.sub_root_keys {
                Some(sub_root_keys) => source_schema.as_ref().is_some_and(|source_schema| {
                    source_schema.sub_schema_url_map.contains_key(sub_root_keys)
                }),
                None => source_schema
                    .as_ref()
                    .is_some_and(|source_schema| source_schema.root_schema.is_some()),
            };
            if already_loaded {
                continue;
            }
            if let Ok(Some(document_schema)) =
                self.try_get_document_schema(&matching_schema.url).await
            {
                match &matching_schema.sub_root_keys {
                    Some(sub_root_keys) => match source_schema {
                        Some(ref mut source_schema) => {
                            if !source_schema.sub_schema_url_map.contains_key(sub_root_keys) {
                                source_schema.sub_schema_url_map.insert(
                                    sub_root_keys.clone(),
                                    document_schema.schema_url.clone(),
                                );
                            }
                        }
                        None => {
                            let mut new_source_schema = SourceSchema {
                                root_schema: None,
                                sub_schema_url_map: Default::default(),
                            };
                            new_source_schema
                                .sub_schema_url_map
                                .insert(sub_root_keys.clone(), document_schema.schema_url.clone());

                            source_schema = Some(new_source_schema);
                        }
                    },
                    None => match source_schema {
                        Some(ref mut source_schema) => {
                            if source_schema.root_schema.is_none() {
                                source_schema.root_schema = Some(document_schema);
                            }
                        }
                        None => {
                            source_schema = Some(SourceSchema {
                                root_schema: Some(document_schema),
                                sub_schema_url_map: Default::default(),
                            });
                        }
                    },
                }
            } else {
                tracing::error!("Can't find matching schema for {}", matching_schema.url);
            }
        }

        Ok(source_schema)
    }

    async fn resolve_source_schema_from_url(
        &self,
        source_url: &url::Url,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        match source_url.scheme() {
            "file" => {
                let source_path = url_to_file_path(source_url).map_err(|_| {
                    crate::Error::SourceUrlParseFailed {
                        source_url: source_url.to_owned(),
                    }
                })?;
                self.resolve_source_schema_from_path(&source_path).await
            }
            "untitled" => Ok(None),
            _ => Err(crate::Error::UnsupportedSourceUrl {
                source_url: source_url.to_owned(),
            }),
        }
    }

    async fn resolve_source_schema(
        &self,
        source_url_or_path: Either<&url::Url, &std::path::Path>,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        match source_url_or_path {
            Either::Left(source_url) => self.resolve_source_schema_from_url(source_url).await,
            Either::Right(source_path) => self.resolve_source_schema_from_path(source_path).await,
        }
        .inspect(|source_schema| {
            if let Some(source_schema) = source_schema {
                if let Some(root_schema) = &source_schema.root_schema {
                    tracing::trace!("find root schema from {}", root_schema.schema_url);
                }
                for (accessors, schema_url) in &source_schema.sub_schema_url_map {
                    tracing::trace!(
                        "find sub schema {:?} from {}",
                        SchemaAccessors::new(accessors.clone()),
                        schema_url
                    );
                }
            }
        })
    }

    pub async fn associate_schema(&self, schema_url: SchemaUrl, include: Vec<String>) {
        let mut schemas = self.schemas.write().await;
        schemas.push(crate::Schema {
            url: schema_url,
            include,
            toml_version: None,
            sub_root_keys: None,
        });
    }
}
