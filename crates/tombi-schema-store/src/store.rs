use std::{borrow::Cow, ops::Deref, str::FromStr, sync::Arc};

use crate::{
    get_tombi_schemastore_content, http_client::HttpClient, json::JsonCatalog, AllOfSchema,
    AnyOfSchema, CatalogUri, DocumentSchema, OneOfSchema, SchemaAccessor, SchemaAccessors,
    SourceSchema, ValueSchema,
};
use ahash::AHashMap;
use itertools::{Either, Itertools};
use tokio::sync::RwLock;
use tombi_ast::SchemaDocumentCommentDirective;
use tombi_cache::{get_cache_file_path, read_from_cache, refresh_cache, save_to_cache};
use tombi_config::{Schema, SchemaOptions};
use tombi_future::{BoxFuture, Boxable};
use tombi_uri::SchemaUri;

#[derive(Debug, Clone)]
pub struct SchemaStore {
    http_client: HttpClient,
    document_schemas:
        Arc<tokio::sync::RwLock<AHashMap<SchemaUri, Result<DocumentSchema, crate::Error>>>>,
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

    pub async fn is_empty(&self) -> bool {
        self.document_schemas.read().await.is_empty() && self.schemas.read().await.is_empty()
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
            self.load_config_schemas(
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
                    let Ok(tagalog_uri) = catalog_path.try_to_catalog_url(base_dir_path) else {
                        return Err(crate::Error::CatalogPathConvertUriFailed {
                            catalog_path: catalog_path.to_string(),
                        });
                    };
                    let tagalog_uri = CatalogUri::from(tagalog_uri);
                    self.load_catalog_from_uri(&tagalog_uri).await
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

    pub async fn load_config_schemas(
        &self,
        schemas: &[Schema],
        base_dir_path: Option<&std::path::Path>,
    ) {
        futures::future::join_all(schemas.iter().map(|schema| async move {
            let schema_uri = if let Ok(schema_uri) = SchemaUri::from_str(schema.path()) {
                schema_uri
            } else if let Ok(schema_uri) = match base_dir_path {
                Some(base_dir_path) => SchemaUri::from_file_path(base_dir_path.join(schema.path())),
                None => SchemaUri::from_file_path(schema.path()),
            } {
                schema_uri
            } else {
                tracing::warn!("Invalid schema path: {}", schema.path());
                return;
            };

            tracing::debug!("Load schema from config: {}", schema_uri);

            self.schemas.write().await.push(crate::Schema {
                url: schema_uri,
                include: schema.include().to_vec(),
                toml_version: schema.toml_version(),
                sub_root_keys: schema.root().and_then(SchemaAccessor::parse),
            });
        }))
        .await;
    }

    pub async fn load_catalog_from_uri(
        &self,
        catalog_uri: &CatalogUri,
    ) -> Result<Option<JsonCatalog>, crate::Error> {
        Ok(Some(match catalog_uri.scheme() {
            "file" => {
                let catalog_path = catalog_uri.to_file_path().map_err(|_| {
                    crate::Error::InvalidCatalogFileUri {
                        catalog_uri: catalog_uri.clone(),
                    }
                })?;

                if !catalog_path.exists() {
                    return Err(crate::Error::CatalogFileNotFound {
                        catalog_path: catalog_path.to_path_buf(),
                    });
                }

                let content = std::fs::read_to_string(&catalog_path).map_err(|_| {
                    crate::Error::CatalogFileReadFailed {
                        catalog_path: catalog_path.to_path_buf(),
                    }
                })?;

                tracing::debug!("load catalog from file: {}", catalog_uri);

                serde_json::from_str(&content).map_err(|err| crate::Error::InvalidJsonFormat {
                    uri: catalog_uri.deref().clone(),
                    reason: err.to_string(),
                })?
            }
            "http" | "https" => {
                let catalog_cache_path = get_cache_file_path(catalog_uri).await;
                if let Some(catalog_cache_path) = &catalog_cache_path {
                    if let Ok(Some(catalog)) = load_catalog_from_cache(
                        catalog_uri,
                        catalog_cache_path,
                        self.options.cache.as_ref(),
                    )
                    .await
                    {
                        return Ok(Some(catalog));
                    }
                }

                if self.offline() {
                    if let Ok(Some(catalog)) = load_catalog_from_cache_ignoring_ttl(
                        catalog_uri,
                        catalog_cache_path.as_deref(),
                        self.options.cache.clone(),
                    )
                    .await
                    {
                        return Ok(Some(catalog));
                    }
                    tracing::debug!("offline mode, skip fetch catalog from url: {}", catalog_uri);
                    return Ok(None);
                }

                let bytes = match self.http_client.get_bytes(catalog_uri.as_str()).await {
                    Ok(bytes) => {
                        tracing::debug!("fetch catalog from url: {}", catalog_uri);
                        bytes
                    }
                    Err(err) => {
                        if let Ok(Some(catalog)) = load_catalog_from_cache_ignoring_ttl(
                            catalog_uri,
                            catalog_cache_path.as_deref(),
                            self.options.cache.clone(),
                        )
                        .await
                        {
                            return Ok(Some(catalog));
                        }
                        return Err(crate::Error::CatalogUriFetchFailed {
                            catalog_uri: catalog_uri.clone(),
                            reason: err.to_string(),
                        });
                    }
                };

                if let Err(err) = save_to_cache(catalog_cache_path.as_deref(), &bytes).await {
                    tracing::warn!("{err}");
                }

                match serde_json::from_slice::<crate::json::JsonCatalog>(&bytes) {
                    Ok(catalog) => catalog,
                    Err(err) => {
                        return Err(crate::Error::InvalidJsonFormat {
                            uri: catalog_uri.deref().clone(),
                            reason: err.to_string(),
                        })
                    }
                }
            }
            "tombi" => {
                let Some(content) = get_tombi_schemastore_content(catalog_uri) else {
                    return Err(crate::Error::InvalidCatalogFileUri {
                        catalog_uri: catalog_uri.clone(),
                    });
                };

                serde_json::from_str::<crate::json::JsonCatalog>(content).map_err(|err| {
                    crate::Error::InvalidJsonFormat {
                        uri: catalog_uri.deref().clone(),
                        reason: err.to_string(),
                    }
                })?
            }
            _ => {
                return Err(crate::Error::UnsupportedUriScheme {
                    uri: catalog_uri.deref().clone(),
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

    pub async fn update_schema(&self, schema_uri: &SchemaUri) -> Result<bool, crate::Error> {
        if matches!(schema_uri.scheme(), "http" | "https") && self.offline() {
            tracing::debug!("offline mode, skip fetch schema from url: {}", schema_uri);
            return Ok(false);
        }

        let has_key = { self.document_schemas.read().await.contains_key(schema_uri) };
        if has_key {
            if let Some(document_schema) = self.fetch_document_schema(schema_uri).await.transpose()
            {
                self.document_schemas
                    .write()
                    .await
                    .insert(schema_uri.clone(), document_schema);
                tracing::debug!("update schema: {}", schema_uri);
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn fetch_schema_value(
        &self,
        schema_uri: &SchemaUri,
    ) -> Result<Option<tombi_json::ValueNode>, crate::Error> {
        match schema_uri.scheme() {
            "file" => {
                let schema_path = tombi_uri::Uri::to_file_path(schema_uri).map_err(|_| {
                    crate::Error::InvalidSchemaUri {
                        schema_uri: schema_uri.to_string(),
                    }
                })?;

                if !schema_path.exists() {
                    return Err(crate::Error::SchemaFileNotFound {
                        schema_path: schema_path.clone(),
                    });
                }

                let file = std::fs::File::open(&schema_path)
                    .map_err(|_| crate::Error::SchemaFileReadFailed { schema_path })?;

                tracing::debug!("fetch schema from file: {}", schema_uri);

                Ok(Some(tombi_json::ValueNode::from_reader(file).map_err(
                    |err| crate::Error::SchemaFileParseFailed {
                        schema_uri: schema_uri.to_owned(),
                        reason: err.to_string(),
                    },
                )?))
            }
            "http" | "https" => {
                let schema_cache_path = get_cache_file_path(schema_uri).await;
                if let Some(schema_cache_path) = &schema_cache_path {
                    if let Ok(Some(schema_value)) = load_json_schema_from_cache(
                        schema_uri,
                        schema_cache_path,
                        self.options.cache.as_ref(),
                    )
                    .await
                    {
                        return Ok(Some(schema_value));
                    }
                }

                if self.offline() {
                    if let Ok(Some(schema_value)) = load_json_schema_from_cache_ignoring_ttl(
                        schema_uri,
                        schema_cache_path.as_deref(),
                        self.options.cache.clone(),
                    )
                    .await
                    {
                        return Ok(Some(schema_value));
                    }
                    tracing::debug!("offline mode, skip fetch schema from uri: {}", schema_uri);
                    return Ok(None);
                }

                let bytes = match self.http_client.get_bytes(schema_uri.as_str()).await {
                    Ok(bytes) => {
                        tracing::debug!("fetch schema from uri: {}", schema_uri);
                        bytes
                    }
                    Err(err) => {
                        if let Ok(Some(schema_value)) = load_json_schema_from_cache_ignoring_ttl(
                            schema_uri,
                            schema_cache_path.as_deref(),
                            self.options.cache.clone(),
                        )
                        .await
                        {
                            return Ok(Some(schema_value));
                        }
                        return Err(crate::Error::SchemaFetchFailed {
                            schema_uri: schema_uri.clone(),
                            reason: err.to_string(),
                        });
                    }
                };

                if let Err(err) = save_to_cache(schema_cache_path.as_deref(), &bytes).await {
                    tracing::warn!("{err}");
                }

                Ok(Some(
                    tombi_json::ValueNode::from_reader(std::io::Cursor::new(bytes)).map_err(
                        |err| crate::Error::SchemaFileParseFailed {
                            schema_uri: schema_uri.to_owned(),
                            reason: err.to_string(),
                        },
                    )?,
                ))
            }
            "tombi" => {
                let Some(content) = get_tombi_schemastore_content(schema_uri) else {
                    return Err(crate::Error::SchemaResourceNotFound {
                        schema_uri: schema_uri.to_owned(),
                    });
                };

                tracing::debug!("fetch schema from embedded file: {}", schema_uri);

                Ok(Some(tombi_json::ValueNode::from_str(content).map_err(
                    |err| crate::Error::SchemaFileParseFailed {
                        schema_uri: schema_uri.to_owned(),
                        reason: err.to_string(),
                    },
                )?))
            }
            _ => Err(crate::Error::UnsupportedUriScheme {
                uri: schema_uri.deref().clone(),
            }),
        }
    }

    async fn fetch_document_schema(
        &self,
        schema_uri: &SchemaUri,
    ) -> Result<Option<DocumentSchema>, crate::Error> {
        let object = match self.fetch_schema_value(schema_uri).await? {
            Some(tombi_json::ValueNode::Object(object)) => object,
            Some(_) => {
                return Err(crate::Error::SchemaMustBeObject {
                    schema_uri: schema_uri.to_owned(),
                });
            }
            None => return Ok(None),
        };
        let document_schema = DocumentSchema::new(object, schema_uri.clone());
        if let Some(
            ValueSchema::AllOf(AllOfSchema { schemas, .. })
            | ValueSchema::AnyOf(AnyOfSchema { schemas, .. })
            | ValueSchema::OneOf(OneOfSchema { schemas, .. }),
        ) = &document_schema.value_schema
        {
            {
                for referable_schema in schemas.write().await.iter_mut() {
                    referable_schema
                        .resolve(
                            Cow::Borrowed(schema_uri),
                            Cow::Borrowed(&document_schema.definitions),
                            self,
                        )
                        .await?;
                }
            }
        }

        Ok(Some(document_schema))
    }

    pub fn try_get_document_schema<'a: 'b, 'b>(
        &'a self,
        schema_uri: &'a SchemaUri,
    ) -> BoxFuture<'b, Result<Option<DocumentSchema>, crate::Error>> {
        async move {
            // Use memory cache first
            if let Some(document_schema) = self.document_schemas.read().await.get(schema_uri) {
                return match document_schema {
                    Ok(document_schema) => Ok(Some(document_schema.clone())),
                    Err(err) => Err(err.to_owned()),
                };
            }

            // Then fetch from remote
            match self.fetch_document_schema(schema_uri).await.transpose() {
                Some(document_schema) => {
                    self.document_schemas
                        .write()
                        .await
                        .insert(schema_uri.clone(), document_schema.clone());

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
        schema_uri: &SchemaUri,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        Ok(Some(SourceSchema {
            root_schema: self.try_get_document_schema(schema_uri).await?,
            sub_schema_uri_map: Default::default(),
        }))
    }

    pub async fn resolve_source_schema_from_ast(
        &self,
        root: &tombi_ast::Root,
        source_uri_or_path: Option<Either<&tombi_uri::Uri, &std::path::Path>>,
    ) -> Result<Option<SourceSchema>, (crate::Error, tombi_text::Range)> {
        let source_path = match source_uri_or_path {
            Some(Either::Left(url)) => match url.scheme() {
                "file" => tombi_uri::Uri::to_file_path(url).ok(),
                _ => None,
            },
            Some(Either::Right(path)) => Some(path.to_path_buf()),
            None => None,
        };

        if let Some(SchemaDocumentCommentDirective { uri, uri_range, .. }) =
            root.schema_document_comment_directive(source_path.as_deref())
        {
            let schema_uri = match uri {
                Ok(schema_uri) => schema_uri,
                Err(schema_uri_or_file_path) => {
                    return Err((
                        crate::Error::InvalidSchemaUriOrFilePath {
                            schema_uri_or_file_path,
                        },
                        uri_range,
                    ));
                }
            };
            return self
                .try_get_source_schema_from_remote_url(&schema_uri)
                .await
                .map_err(|err| (err, uri_range));
        }

        if let Some(source_uri_or_path) = source_uri_or_path {
            Ok(self
                .resolve_source_schema(source_uri_or_path)
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
                        format!("**/{pat}")
                    } else {
                        pat.to_string()
                    };
                    glob::Pattern::new(&pattern)
                        .ok()
                        .map(|glob_pat| glob_pat.matches_path(source_path))
                        .unwrap_or(false)
                })
            })
            .collect_vec();

        let mut source_schema: Option<SourceSchema> = None;
        for matching_schema in matching_schemas {
            // Skip if the same schema (by URL and sub_root_keys) is already loaded in source_schema
            let already_loaded = match &matching_schema.sub_root_keys {
                Some(sub_root_keys) => source_schema.as_ref().is_some_and(|source_schema| {
                    source_schema.sub_schema_uri_map.contains_key(sub_root_keys)
                }),
                None => source_schema
                    .as_ref()
                    .is_some_and(|source_schema| source_schema.root_schema.is_some()),
            };
            if already_loaded {
                continue;
            }
            match self.try_get_document_schema(&matching_schema.url).await {
                Ok(Some(document_schema)) => match &matching_schema.sub_root_keys {
                    Some(sub_root_keys) => match source_schema {
                        Some(ref mut source_schema) => {
                            if !source_schema.sub_schema_uri_map.contains_key(sub_root_keys) {
                                source_schema.sub_schema_uri_map.insert(
                                    sub_root_keys.clone(),
                                    document_schema.schema_uri.clone(),
                                );
                            }
                        }
                        None => {
                            let mut new_source_schema = SourceSchema {
                                root_schema: None,
                                sub_schema_uri_map: Default::default(),
                            };
                            new_source_schema
                                .sub_schema_uri_map
                                .insert(sub_root_keys.clone(), document_schema.schema_uri.clone());

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
                                sub_schema_uri_map: Default::default(),
                            });
                        }
                    },
                },
                Ok(None) => {
                    tracing::warn!("Failed to find document schema: {}", matching_schema.url);
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to get document schema for {url}: {err}",
                        url = matching_schema.url,
                    );
                }
            }
        }

        Ok(source_schema)
    }

    async fn resolve_source_schema_from_uri(
        &self,
        source_uri: &tombi_uri::Uri,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        match source_uri.scheme() {
            "file" => {
                let source_path = tombi_uri::Uri::to_file_path(source_uri).map_err(|_| {
                    crate::Error::SourceUriParseFailed {
                        source_uri: source_uri.to_owned(),
                    }
                })?;
                self.resolve_source_schema_from_path(&source_path).await
            }
            "untitled" => Ok(None),
            _ => Err(crate::Error::UnsupportedSourceUri {
                source_uri: source_uri.to_owned(),
            }),
        }
    }

    pub(crate) async fn resolve_source_schema(
        &self,
        source_uri_or_path: Either<&tombi_uri::Uri, &std::path::Path>,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        match source_uri_or_path {
            Either::Left(source_uri) => self.resolve_source_schema_from_uri(source_uri).await,
            Either::Right(source_path) => self.resolve_source_schema_from_path(source_path).await,
        }
        .inspect(|source_schema| {
            if let Some(source_schema) = source_schema {
                if let Some(root_schema) = &source_schema.root_schema {
                    tracing::trace!("find root schema from {}", root_schema.schema_uri);
                }
                for (accessors, schema_uri) in &source_schema.sub_schema_uri_map {
                    tracing::trace!(
                        "find sub schema {:?} from {}",
                        SchemaAccessors::from(accessors.clone()),
                        schema_uri
                    );
                }
            }
        })
    }

    pub async fn associate_schema(&self, schema_uri: SchemaUri, include: Vec<String>) {
        let mut schemas = self.schemas.write().await;
        schemas.push(crate::Schema {
            url: schema_uri,
            include,
            toml_version: None,
            sub_root_keys: None,
        });
    }
}

async fn load_catalog_from_cache_ignoring_ttl(
    tagalog_uri: &CatalogUri,
    catalog_cache_path: Option<&std::path::Path>,
    cache_options: Option<tombi_cache::Options>,
) -> Result<Option<JsonCatalog>, crate::Error> {
    if let Some(catalog_cache_path) = catalog_cache_path {
        let mut cache_options = cache_options.clone();
        if let Some(options) = &mut cache_options {
            options.cache_ttl = None;
        }
        if let Ok(Some(catalog)) =
            load_catalog_from_cache(tagalog_uri, catalog_cache_path, cache_options.as_ref()).await
        {
            return Ok(Some(catalog));
        }
    }

    Ok(None)
}

async fn load_catalog_from_cache(
    tagalog_uri: &CatalogUri,
    catalog_cache_path: &std::path::Path,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<JsonCatalog>, crate::Error> {
    if let Some(catalog_cache_content) =
        read_from_cache(Some(catalog_cache_path), cache_options).await?
    {
        tracing::debug!("load catalog from cache: {}", tagalog_uri);

        return Ok(Some(serde_json::from_str(&catalog_cache_content).map_err(
            |err| crate::Error::CatalogFileParseFailed {
                tagalog_uri: tagalog_uri.to_owned(),
                reason: err.to_string(),
            },
        )?));
    }

    Ok(None)
}

/// Attempt to load the json schema from the cache, ignoring the TTL.
async fn load_json_schema_from_cache_ignoring_ttl(
    schema_uri: &SchemaUri,
    schema_cache_path: Option<&std::path::Path>,
    cache_options: Option<tombi_cache::Options>,
) -> Result<Option<tombi_json::ValueNode>, crate::Error> {
    if let Some(schema_cache_path) = schema_cache_path {
        let mut cache_options = cache_options.clone();
        if let Some(options) = &mut cache_options {
            options.cache_ttl = None;
        }
        if let Ok(Some(schema_value)) =
            load_json_schema_from_cache(schema_uri, schema_cache_path, cache_options.as_ref()).await
        {
            return Ok(Some(schema_value));
        }
    }

    Ok(None)
}

async fn load_json_schema_from_cache(
    schema_uri: &SchemaUri,
    schema_cache_path: &std::path::Path,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<tombi_json::ValueNode>, crate::Error> {
    if let Some(schema_cache_content) =
        read_from_cache(Some(schema_cache_path), cache_options).await?
    {
        tracing::debug!("load schema from cache: {}", schema_uri);

        return Ok(Some(
            tombi_json::ValueNode::from_str(&schema_cache_content).map_err(|err| {
                crate::Error::SchemaFileParseFailed {
                    schema_uri: schema_uri.to_owned(),
                    reason: err.to_string(),
                }
            })?,
        ));
    }

    Ok(None)
}
