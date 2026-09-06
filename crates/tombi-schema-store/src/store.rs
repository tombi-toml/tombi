use std::{
    borrow::Cow,
    cell::RefCell,
    ops::Deref,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::resolve_json_pointer;
use crate::{
    AllOfSchema, AnyOfSchema, CatalogUri, DocumentSchema, OneOfSchema, PatternAccessor,
    PatternAccessors, ResourceIndex, SchemaAnchors, SchemaDynamicAnchors, SchemaMap, SchemaView,
    SourceSchema, SubSchemaLink, SubSchemaLinkMap, get_tombi_schemastore_content,
    http_client::{DefaultHttpClient, HttpClient},
    json::JsonCatalog,
};
use itertools::{Either, Itertools};
use tokio::sync::RwLock;
#[cfg(feature = "ast-syntax")]
use tombi_ast_syntax::SchemaDocumentCommentDirective;
use tombi_cache::{get_cache_file_path, read_from_cache, refresh_cache, save_to_cache};
use tombi_config::{SchemaItem, SchemaOverviewOptions, TomlVersion, config_base_dir};
use tombi_future::{BoxFuture, Boxable};
use tombi_uri::SchemaUri;

type SchemaCache = Arc<RwLock<CacheState>>;

#[derive(Debug, Default)]
struct CacheState {
    documents: tombi_hashmap::HashMap<SchemaUri, CachedDocumentSchema>,
    resources: ResourceRegistry,
}

#[derive(Debug, Default)]
struct ResourceRegistry {
    by_uri: tombi_hashmap::HashMap<SchemaUri, EmbeddedResource>,
    by_source: tombi_hashmap::HashMap<SchemaUri, Arc<SchemaGeneration>>,
}

tokio::task_local! {
    static RESOLUTION_STACK: RefCell<Vec<SchemaUri>>;
}

struct ResolutionGuard(SchemaUri);

impl ResolutionGuard {
    fn enter(schema_uri: &SchemaUri) -> Option<Self> {
        RESOLUTION_STACK
            .try_with(|stack| {
                let mut stack = stack.borrow_mut();
                if stack.contains(schema_uri) {
                    None
                } else {
                    stack.push(schema_uri.clone());
                    Some(Self(schema_uri.clone()))
                }
            })
            .ok()
            .flatten()
    }
}

impl Drop for ResolutionGuard {
    fn drop(&mut self) {
        let _ = RESOLUTION_STACK.try_with(|stack| {
            let mut stack = stack.borrow_mut();
            if let Some(position) = stack.iter().rposition(|uri| uri == &self.0) {
                stack.remove(position);
            }
        });
    }
}

#[derive(Debug, Clone)]
struct EmbeddedResource {
    generation: Arc<SchemaGeneration>,
}

#[derive(Debug)]
pub(crate) struct SchemaGeneration {
    revision: u64,
    pub(crate) source_schema_uri: Arc<SchemaUri>,
    pub(crate) source_node: Arc<tombi_json::ValueNode>,
    pub(crate) resource_index: Arc<ResourceIndex>,
    /// Completed resource schemas with all generation-local strong references
    /// removed. Misses may compile concurrently; insertion is idempotent.
    compiled_resources: std::sync::RwLock<tombi_hashmap::HashMap<SchemaUri, Arc<DocumentSchema>>>,
}

fn build_schema_map(
    schema_value: &tombi_json::ValueNode,
    targets: &tombi_hashmap::HashMap<String, crate::ResourceTarget>,
    string_formats: Option<&[tombi_x_keyword::StringFormat]>,
    dialect: Option<crate::JsonSchemaDialect>,
) -> SchemaMap {
    targets
        .iter()
        .filter_map(|(name, target)| {
            crate::resolve_json_pointer_node(schema_value, &target.pointer)
                .and_then(|node| {
                    crate::schema::referable_from_schema_value(
                        node,
                        string_formats,
                        dialect,
                        None,
                        None,
                    )
                })
                .map(|schema| (name.clone(), schema))
        })
        .collect()
}

fn build_anchor_maps(
    schema_value: &tombi_json::ValueNode,
    metadata: &crate::ResourceMetadata,
    string_formats: Option<&[tombi_x_keyword::StringFormat]>,
    dialect: Option<crate::JsonSchemaDialect>,
) -> (SchemaAnchors, SchemaDynamicAnchors) {
    (
        SchemaAnchors::new(RwLock::new(build_schema_map(
            schema_value,
            &metadata.anchors,
            string_formats,
            dialect,
        ))),
        SchemaDynamicAnchors::new(RwLock::new(build_schema_map(
            schema_value,
            &metadata.dynamic_anchors,
            string_formats,
            dialect,
        ))),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SchemaCacheVersion {
    modified_at_nanos: u64,
    len: u64,
}

#[derive(Debug, Clone)]
struct CachedDocumentSchema {
    version: Option<SchemaCacheVersion>,
    source_schema_uri: SchemaUri,
    document_schema: Result<Arc<DocumentSchema>, crate::Error>,
}

#[derive(Debug, Clone)]
struct StoredSchema {
    schema: crate::Schema,
    patterns: SchemaPatterns,
}

#[derive(Debug, Clone)]
struct SchemaPatterns {
    include_patterns: Vec<glob::Pattern>,
    exclude_patterns: Vec<glob::Pattern>,
}

impl StoredSchema {
    fn new(schema: crate::Schema) -> Self {
        let patterns = SchemaPatterns::new(&schema.include, schema.exclude.as_deref());

        Self { schema, patterns }
    }

    fn matches(
        &self,
        path_for_matching: &std::path::Path,
        absolute_source_path: &std::path::Path,
    ) -> bool {
        self.patterns
            .matches(path_for_matching, absolute_source_path)
    }
}

impl SchemaPatterns {
    fn new(include: &[String], exclude: Option<&[String]>) -> Self {
        Self {
            include_patterns: compile_schema_patterns(include),
            exclude_patterns: exclude.map(compile_schema_patterns).unwrap_or_default(),
        }
    }

    fn matches(
        &self,
        path_for_matching: &std::path::Path,
        absolute_source_path: &std::path::Path,
    ) -> bool {
        let matches_path = |pattern: &glob::Pattern| {
            pattern.matches_path(path_for_matching)
                || (path_for_matching != absolute_source_path
                    && pattern.matches_path(absolute_source_path))
        };

        self.include_patterns.iter().any(matches_path)
            && !self.exclude_patterns.iter().any(matches_path)
    }
}

impl Deref for StoredSchema {
    type Target = crate::Schema;

    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}

/// Options for associating a schema with file patterns
#[derive(Debug, Clone, Default)]
pub struct AssociateSchemaOptions {
    pub title: Option<String>,
    pub description: Option<String>,
    pub toml_version: Option<TomlVersion>,
    /// If true, the schema will be inserted at the beginning to force precedence
    pub force: bool,
}

#[derive(Debug, Clone)]
pub struct SchemaStore {
    http_client: Arc<dyn HttpClient>,
    cache: SchemaCache,
    next_generation: Arc<AtomicU64>,
    schemas: Arc<RwLock<Vec<StoredSchema>>>,
    options: crate::Options,
    base_dir_path: Arc<RwLock<Option<std::path::PathBuf>>>,
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
        self.cache.read().await.documents.is_empty() && self.schemas.read().await.is_empty()
    }

    /// New with options
    ///
    /// Create a store with the given options.
    /// Note that the new_with_options() does not automatically load schemas from Config etc.
    pub fn new_with_options(options: crate::Options) -> Self {
        Self::new_with_options_and_http_client(options, Arc::new(DefaultHttpClient::new()))
    }

    /// Create a store with a caller-provided HTTP client.
    pub fn new_with_options_and_http_client(
        options: crate::Options,
        http_client: Arc<dyn HttpClient>,
    ) -> Self {
        Self {
            http_client,
            cache: Arc::new(RwLock::default()),
            next_generation: Arc::new(AtomicU64::new(0)),
            schemas: Arc::new(RwLock::new(Vec::new())),
            options,
            base_dir_path: Arc::new(RwLock::new(None)),
        }
    }

    /// Offline mode
    pub fn offline(&self) -> bool {
        self.options.offline.unwrap_or_default()
    }

    /// Cache options
    pub fn cache_options(&self) -> Option<&tombi_cache::Options> {
        self.options.cache.as_ref()
    }

    /// Strict mode in global level.
    pub fn strict(&self) -> Option<tombi_schema_type::BoolDefaultTrue> {
        self.options.strict
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
        *self.cache.write().await = CacheState::default();
        self.schemas.write().await.clear();
        self.load_config(config, config_path).await?;
        Ok(())
    }

    pub async fn load_config(
        &self,
        config: &tombi_config::Config,
        config_path: Option<&std::path::Path>,
    ) -> Result<(), crate::Error> {
        let base_dir_path_buf = config_path
            .and_then(config_base_dir)
            .map(canonicalize_path_for_matching);
        let base_dir_path = base_dir_path_buf.as_deref();

        // Set the base directory for schema matching
        *self.base_dir_path.write().await = base_dir_path_buf.clone();

        let schema_options = match &config.schema {
            Some(schema) => schema,
            None => &SchemaOverviewOptions::default(),
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
                    let Ok(catalog_uri) = catalog_path
                        .try_to_catalog_url(base_dir_path)
                        .map(CatalogUri::from)
                    else {
                        return Err(crate::Error::CatalogPathConvertUriFailed {
                            catalog_path: catalog_path.to_string(),
                        });
                    };
                    let catalog_uri = Arc::new(catalog_uri);
                    self.load_catalog_from_uri(&catalog_uri)
                        .await
                        .map(|catalog| catalog.map(|catalog| (catalog_uri.clone(), catalog)))
                }))
                .await;

            for catalog_result in catalogs_results {
                match catalog_result {
                    Ok(Some((catalog_uri, catalog))) => {
                        self.add_json_catalog(catalog_uri, catalog).await?;
                    }
                    Ok(None) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(())
    }

    fn normalize_schema_uri_key(schema_uri: &SchemaUri) -> SchemaUri {
        let mut normalized = schema_uri.clone();
        normalized.set_fragment(None);
        normalized
    }

    async fn load_config_schemas(
        &self,
        schemas: &[SchemaItem],
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
                log::warn!("invalid schema path: {}", schema.path());
                return;
            };

            log::debug!("load schema from config: {}", schema_uri);

            self.schemas
                .write()
                .await
                .push(StoredSchema::new(crate::Schema {
                    title: None,
                    description: None,
                    deprecated_lint_level: schema.deprecated_lint_level(),
                    format_rules: schema.format().and_then(|format| format.rules.clone()),
                    lint_rules: schema.lint().and_then(|lint| lint.rules.clone()),
                    overrides: schema_overrides(schema),
                    strict: schema.strict(),
                    schema_uri,
                    catalog_uri: None,
                    include: schema.include().to_vec(),
                    exclude: schema.exclude().map(|exclude| exclude.to_vec()),
                    toml_version: schema.toml_version(),
                    sub_root_accessors: schema.root().and_then(PatternAccessor::parse),
                }));
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

                log::debug!("load catalog from file: {}", catalog_uri);

                serde_json::from_str(&content).map_err(|err| crate::Error::InvalidJsonFormat {
                    uri: catalog_uri.deref().clone(),
                    reason: err.to_string(),
                })?
            }
            "http" | "https" => {
                let catalog_cache_path = get_cache_file_path(catalog_uri).await;
                if let Some(catalog_cache_path) = &catalog_cache_path
                    && let Ok(Some(catalog)) = load_catalog_from_cache(
                        catalog_uri,
                        catalog_cache_path,
                        self.options.cache.as_ref(),
                    )
                    .await
                {
                    return Ok(Some(catalog));
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
                    log::debug!("offline mode, skip fetch catalog from url: {}", catalog_uri);
                    return Ok(None);
                }

                let bytes = match self.http_client.get_bytes(catalog_uri.as_str()).await {
                    Ok(bytes) => {
                        log::debug!("fetch catalog from url: {}", catalog_uri);
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
                    log::warn!("{err}");
                }

                match serde_json::from_slice::<crate::json::JsonCatalog>(&bytes) {
                    Ok(catalog) => catalog,
                    Err(err) => {
                        return Err(crate::Error::InvalidJsonFormat {
                            uri: catalog_uri.deref().clone(),
                            reason: err.to_string(),
                        });
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

    async fn add_json_catalog(
        &self,
        catalog_uri: Arc<CatalogUri>,
        json_catalog: JsonCatalog,
    ) -> Result<(), crate::Error> {
        let mut schemas = self.schemas.write().await;
        for schema in json_catalog.schemas {
            if schema
                .file_match
                .iter()
                .any(|pattern| pattern.ends_with(".toml"))
            {
                schemas.push(StoredSchema::new(crate::Schema {
                    title: Some(schema.name),
                    description: Some(schema.description),
                    deprecated_lint_level: None,
                    format_rules: None,
                    lint_rules: None,
                    overrides: Default::default(),
                    strict: None,
                    schema_uri: schema.url,
                    catalog_uri: Some(catalog_uri.clone()),
                    include: schema.file_match,
                    exclude: None,
                    toml_version: None,
                    sub_root_accessors: None,
                }));
            }
        }
        Ok(())
    }

    pub async fn update_schema(&self, mut schema_uri: SchemaUri) -> Result<bool, crate::Error> {
        if matches!(schema_uri.scheme(), "http" | "https") && self.offline() {
            log::debug!("offline mode, skip fetch schema from url: {}", schema_uri);
            return Ok(false);
        }

        if schema_uri.fragment().is_some() {
            schema_uri.set_fragment(None);
        }

        let has_key = { self.cache.read().await.documents.contains_key(&schema_uri) };
        if has_key
            && let Some((document_schema, version)) = RESOLUTION_STACK
                .scope(
                    RefCell::new(Vec::new()),
                    self.fetch_stable_document_schema(&schema_uri),
                )
                .await?
        {
            self.replace_cached_source_document(schema_uri.clone(), version, document_schema)
                .await?;
            log::debug!("update schema: {}", schema_uri);
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn fetch_schema_value(
        &self,
        schema_uri: &SchemaUri,
    ) -> Result<Option<tombi_json::ValueNode>, crate::Error> {
        self.fetch_schema_value_with_registry(schema_uri, true)
            .await
    }

    async fn fetch_schema_value_with_registry(
        &self,
        schema_uri: &SchemaUri,
        consult_embedded_registry: bool,
    ) -> Result<Option<tombi_json::ValueNode>, crate::Error> {
        if consult_embedded_registry
            && let Some(resource) = self
                .cache
                .read()
                .await
                .resources
                .by_uri
                .get(schema_uri)
                .cloned()
        {
            let Some(metadata) = resource.generation.resource_index.resource(schema_uri) else {
                return Ok(None);
            };
            return Ok(crate::resolve_json_pointer_node(
                &resource.generation.source_node,
                &metadata.root_pointer,
            )
            .cloned());
        }
        let aliased_source = if consult_embedded_registry {
            self.cache
                .read()
                .await
                .documents
                .get(schema_uri)
                .map(|cached| cached.source_schema_uri.clone())
                .filter(|source| source != schema_uri)
        } else {
            None
        };
        let schema_uri = aliased_source.as_ref().unwrap_or(schema_uri);
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

                log::debug!("load schema from file: {}", schema_uri);

                Ok(Some(tombi_json::ValueNode::from_reader(file).map_err(
                    |err| crate::Error::SchemaFileParseFailed {
                        schema_uri: schema_uri.to_owned(),
                        reason: err.to_string(),
                    },
                )?))
            }
            "http" | "https" => {
                let schema_cache_path = get_cache_file_path(schema_uri).await;
                if let Some(schema_cache_path) = &schema_cache_path
                    && let Ok(Some(schema_value)) = load_json_schema_from_cache(
                        schema_uri,
                        schema_cache_path,
                        self.options.cache.as_ref(),
                    )
                    .await
                {
                    return Ok(Some(schema_value));
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
                    log::debug!("offline mode, skip fetch schema from uri: {}", schema_uri);
                    return Ok(None);
                }

                let bytes = match self.http_client.get_bytes(schema_uri.as_str()).await {
                    Ok(bytes) => {
                        log::debug!("fetch schema from uri: {}", schema_uri);
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
                    log::warn!("{err}");
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

                log::trace!("load schema from embedded file: {}", schema_uri);

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

    pub(crate) async fn fetch_external_schema_value(
        &self,
        schema_uri: &SchemaUri,
    ) -> Result<Option<tombi_json::ValueNode>, crate::Error> {
        self.fetch_schema_value_with_registry(schema_uri, false)
            .await
    }

    async fn fetch_document_schema(
        &self,
        schema_uri: &SchemaUri,
    ) -> Result<Option<Arc<DocumentSchema>>, crate::Error> {
        self.fetch_document_schema_with_registry(schema_uri, true, true)
            .await
    }

    async fn fetch_stable_document_schema(
        &self,
        schema_uri: &SchemaUri,
    ) -> Result<Option<(Arc<DocumentSchema>, Option<SchemaCacheVersion>)>, crate::Error> {
        self.fetch_stable_document_schema_with_registry(schema_uri, true, true)
            .await
    }

    async fn fetch_stable_document_schema_with_registry(
        &self,
        schema_uri: &SchemaUri,
        consult_embedded_registry: bool,
        publish_embedded_resources: bool,
    ) -> Result<Option<(Arc<DocumentSchema>, Option<SchemaCacheVersion>)>, crate::Error> {
        for _ in 0..3 {
            let before = schema_cache_version(schema_uri).await;
            let document_schema = self
                .fetch_document_schema_with_registry(
                    schema_uri,
                    consult_embedded_registry,
                    publish_embedded_resources,
                )
                .await?;
            let after = schema_cache_version(schema_uri).await;
            if before == after {
                return Ok(document_schema.map(|document| (document, after)));
            }
        }

        // Do not bless a racing read with a version it may not represent. A
        // missing version forces the next lookup to retry once the source is stable.
        Ok(self
            .fetch_document_schema_with_registry(
                schema_uri,
                consult_embedded_registry,
                publish_embedded_resources,
            )
            .await?
            .map(|document| (document, None)))
    }

    async fn fetch_document_schema_with_registry(
        &self,
        schema_uri: &SchemaUri,
        consult_embedded_registry: bool,
        publish_embedded_resources: bool,
    ) -> Result<Option<Arc<DocumentSchema>>, crate::Error> {
        if consult_embedded_registry
            && let Some(resource) = self
                .cache
                .read()
                .await
                .resources
                .by_uri
                .get(schema_uri)
                .cloned()
        {
            return self
                .document_schema_from_generation(schema_uri, resource.generation)
                .await;
        }

        let revision = self.next_generation.fetch_add(1, Ordering::Relaxed);
        let schema_value = Arc::new(
            match self
                .fetch_schema_value_with_registry(schema_uri, consult_embedded_registry)
                .await?
            {
                Some(value) => value,
                None => return Ok(None),
            },
        );
        if !matches!(
            schema_value.as_ref(),
            tombi_json::ValueNode::Object(_) | tombi_json::ValueNode::Bool(_)
        ) {
            return Err(crate::Error::SchemaMustBeObjectOrBoolean {
                schema_uri: schema_uri.clone(),
            });
        }
        let resource_index = Arc::new(
            ResourceIndex::build(&schema_value, schema_uri, None).map_err(|error| {
                crate::Error::InvalidSchemaResources {
                    schema_uri: schema_uri.clone(),
                    reason: error.to_string(),
                }
            })?,
        );
        let generation = Arc::new(SchemaGeneration {
            revision,
            source_schema_uri: Arc::new(schema_uri.clone()),
            source_node: schema_value.clone(),
            resource_index: resource_index.clone(),
            compiled_resources: std::sync::RwLock::new(Default::default()),
        });
        let root_resource_uri = resource_index.root_resource_uri().clone();
        let Some(_resolution_guard) = ResolutionGuard::enter(&root_resource_uri) else {
            return Err(crate::Error::CyclicSchemaReference {
                schema_uri: root_resource_uri,
            });
        };
        let mut document_schema = DocumentSchema::new_indexed(
            &schema_value,
            schema_uri.clone(),
            Some(root_resource_uri.clone()),
            None,
            Some(generation.clone()),
            None,
            self,
        )
        .await;
        if let Some(metadata) = resource_index.resource(&root_resource_uri) {
            let (anchors, dynamic_anchors) = build_anchor_maps(
                &schema_value,
                metadata,
                document_schema.string_formats(),
                document_schema.dialect(),
            );
            document_schema.anchors = anchors;
            document_schema.dynamic_anchors = dynamic_anchors;
        }
        if let Some(
            SchemaView::AllOf(AllOfSchema { schemas, .. })
            | SchemaView::AnyOf(AnyOfSchema { schemas, .. })
            | SchemaView::OneOf(OneOfSchema { schemas, .. }),
        ) = document_schema.schema_view.as_deref()
        {
            let document_base_uri = document_schema.base_uri().clone();
            {
                for referable_schema in schemas.write().await.iter_mut() {
                    referable_schema
                        .resolve(
                            Cow::Borrowed(&document_base_uri),
                            Cow::Borrowed(&document_schema.definitions),
                            None,
                            self,
                        )
                        .await?;
                }
            }
        }

        if publish_embedded_resources {
            self.register_embedded_resources(generation).await?;
        }

        Ok(Some(Arc::new(document_schema)))
    }

    pub(crate) async fn try_get_document_schema_in_generation(
        &self,
        schema_uri: &SchemaUri,
        generation: Option<&Arc<SchemaGeneration>>,
    ) -> Result<Option<Arc<DocumentSchema>>, crate::Error> {
        if let Some(generation) = generation {
            if generation.resource_index.resource(schema_uri).is_some() {
                if RESOLUTION_STACK.try_with(|_| ()).is_err() {
                    return RESOLUTION_STACK
                        .scope(
                            RefCell::new(Vec::new()),
                            self.document_schema_from_generation(schema_uri, generation.clone()),
                        )
                        .await;
                }
                return self
                    .document_schema_from_generation(schema_uri, generation.clone())
                    .await;
            }
            let registered = self
                .cache
                .read()
                .await
                .resources
                .by_uri
                .get(schema_uri)
                .filter(|resource| {
                    resource.generation.source_schema_uri != generation.source_schema_uri
                })
                .cloned();
            if let Some(resource) = registered {
                if RESOLUTION_STACK.try_with(|_| ()).is_err() {
                    return RESOLUTION_STACK
                        .scope(
                            RefCell::new(Vec::new()),
                            self.document_schema_from_generation(schema_uri, resource.generation),
                        )
                        .await;
                }
                return self
                    .document_schema_from_generation(schema_uri, resource.generation)
                    .await;
            }
            let cached_source = self
                .cache
                .read()
                .await
                .documents
                .get(schema_uri)
                .map(|cached| cached.source_schema_uri.clone());
            if cached_source
                .as_ref()
                .is_some_and(|source| source != generation.source_schema_uri.as_ref())
                && let Some(document_schema) = self.try_get_document_schema(schema_uri).await?
                && document_schema
                    .definitions
                    .generation()
                    .is_none_or(|resolved_generation| {
                        resolved_generation.source_schema_uri != generation.source_schema_uri
                    })
            {
                return Ok(Some(document_schema));
            }
            if RESOLUTION_STACK.try_with(|_| ()).is_err() {
                return RESOLUTION_STACK
                    .scope(
                        RefCell::new(Vec::new()),
                        self.fetch_and_cache_external_document_schema(schema_uri, generation),
                    )
                    .await;
            }
            self.fetch_and_cache_external_document_schema(schema_uri, generation)
                .await
        } else {
            self.try_get_document_schema(schema_uri).await
        }
    }

    async fn fetch_and_cache_external_document_schema(
        &self,
        schema_uri: &SchemaUri,
        pinned_generation: &Arc<SchemaGeneration>,
    ) -> Result<Option<Arc<DocumentSchema>>, crate::Error> {
        let Some((document_schema, version)) = self
            .fetch_stable_document_schema_with_registry(schema_uri, false, false)
            .await?
        else {
            return Ok(None);
        };
        if self
            .try_publish_external_candidate(schema_uri, version, document_schema.clone())
            .await
        {
            return Ok(Some(document_schema));
        }

        let registered = self
            .cache
            .read()
            .await
            .resources
            .by_uri
            .get(schema_uri)
            .filter(|resource| {
                resource.generation.source_schema_uri != pinned_generation.source_schema_uri
            })
            .cloned();
        if let Some(resource) = registered {
            return self
                .document_schema_from_generation(schema_uri, resource.generation)
                .await;
        }
        let cached_source = self
            .cache
            .read()
            .await
            .documents
            .get(schema_uri)
            .map(|cached| cached.source_schema_uri.clone());
        if cached_source
            .as_ref()
            .is_some_and(|source| source != pinned_generation.source_schema_uri.as_ref())
            && let Some(document_schema) = self.try_get_document_schema(schema_uri).await?
            && document_schema
                .definitions
                .generation()
                .is_none_or(|resolved_generation| {
                    resolved_generation.source_schema_uri != pinned_generation.source_schema_uri
                })
        {
            return Ok(Some(document_schema));
        }
        // A pinned generation may legitimately need a private external fallback
        // whose URI is owned globally by a newer same-source generation.
        log::debug!("keep external schema private: {schema_uri}");
        Ok(Some(document_schema))
    }

    /// Publishes a separately-built external candidate as one transaction.
    /// Returning `false` leaves both registries unchanged.
    async fn try_publish_external_candidate(
        &self,
        key: &SchemaUri,
        version: Option<SchemaCacheVersion>,
        document_schema: Arc<DocumentSchema>,
    ) -> bool {
        let Some(generation) = document_schema.owner_generation.clone() else {
            return false;
        };
        let source_schema_uri = generation.source_schema_uri.as_ref().clone();
        let resource_uris = generation
            .resource_index
            .resources()
            .keys()
            .filter(|uri| {
                *uri != generation.resource_index.root_resource_uri() && *uri != &source_schema_uri
            })
            .cloned()
            .collect_vec();
        let aliases = std::iter::once(key)
            .chain(document_schema.id.as_ref())
            .collect_vec();

        let mut cache = self.cache.write().await;
        if cache
            .resources
            .by_source
            .get(&source_schema_uri)
            .is_some_and(|installed| installed.revision > generation.revision)
        {
            return false;
        }
        if resource_uris.iter().any(|uri| {
            cache
                .documents
                .get(uri)
                .is_some_and(|existing| existing.source_schema_uri != source_schema_uri)
                || cache.resources.by_uri.get(uri).is_some_and(|existing| {
                    existing.generation.source_schema_uri.as_ref() != &source_schema_uri
                })
        }) || aliases.iter().any(|uri| {
            cache
                .documents
                .get(*uri)
                .is_some_and(|existing| existing.source_schema_uri != source_schema_uri)
                || cache.resources.by_uri.get(*uri).is_some_and(|existing| {
                    existing.generation.source_schema_uri.as_ref() != &source_schema_uri
                })
        }) {
            return false;
        }

        cache.resources.by_uri.retain(|_, resource| {
            resource.generation.source_schema_uri.as_ref() != &source_schema_uri
        });
        cache
            .resources
            .by_uri
            .extend(resource_uris.into_iter().map(|uri| {
                (
                    uri,
                    EmbeddedResource {
                        generation: generation.clone(),
                    },
                )
            }));
        cache
            .resources
            .by_source
            .insert(source_schema_uri.clone(), generation);

        cache
            .documents
            .retain(|_, existing| existing.source_schema_uri != source_schema_uri);
        let id = document_schema.id.clone();
        let cached = CachedDocumentSchema {
            version,
            source_schema_uri: source_schema_uri.clone(),
            document_schema: Ok(document_schema),
        };
        cache.documents.insert(key.clone(), cached.clone());
        if let Some(id) = id {
            cache.documents.insert(id, cached);
        }
        true
    }

    async fn document_schema_from_generation(
        &self,
        schema_uri: &SchemaUri,
        generation: Arc<SchemaGeneration>,
    ) -> Result<Option<Arc<DocumentSchema>>, crate::Error> {
        let compiled = {
            generation
                .compiled_resources
                .read()
                .expect("compiled resource cache poisoned")
                .get(schema_uri)
                .cloned()
        };
        if let Some(template) = compiled {
            let mut document_schema = template.as_ref().clone();
            document_schema.definitions = document_schema
                .definitions
                .with_generation(generation.clone());
            document_schema.owner_generation = Some(generation);
            return Ok(Some(Arc::new(document_schema)));
        }
        let Some(_resolution_guard) = ResolutionGuard::enter(schema_uri) else {
            return Err(crate::Error::CyclicSchemaReference {
                schema_uri: schema_uri.clone(),
            });
        };
        let Some(metadata) = generation.resource_index.resource(schema_uri) else {
            return Ok(None);
        };
        let Some(node) =
            crate::resolve_json_pointer_node(&generation.source_node, &metadata.root_pointer)
        else {
            return Ok(None);
        };
        let mut document_schema = DocumentSchema::new_embedded(
            node,
            generation.source_schema_uri.as_ref().clone(),
            schema_uri.clone(),
            metadata.dialect,
            generation.clone(),
            None,
            self,
        )
        .await;
        let (anchors, dynamic_anchors) = build_anchor_maps(
            &generation.source_node,
            metadata,
            document_schema.string_formats(),
            document_schema.dialect(),
        );
        document_schema.anchors = anchors;
        document_schema.dynamic_anchors = dynamic_anchors;
        if document_schema.definitions.dynamic_scope().is_empty()
            && document_schema
                .definitions
                .generation()
                .is_some_and(|resolved_generation| Arc::ptr_eq(resolved_generation, &generation))
        {
            let mut template = document_schema.clone();
            template.definitions = template.definitions.without_runtime_context();
            template.owner_generation = None;
            generation
                .compiled_resources
                .write()
                .expect("compiled resource cache poisoned")
                .entry(schema_uri.clone())
                .or_insert_with(|| Arc::new(template));
        }
        Ok(Some(Arc::new(document_schema)))
    }

    async fn register_embedded_resources(
        &self,
        generation: Arc<SchemaGeneration>,
    ) -> Result<bool, crate::Error> {
        let mut discovered = Vec::new();
        let source_schema_uri = &generation.source_schema_uri;
        for canonical_uri in generation.resource_index.resources().keys() {
            if canonical_uri == generation.resource_index.root_resource_uri()
                && canonical_uri == source_schema_uri.as_ref()
            {
                continue;
            }
            discovered.push((
                canonical_uri.clone(),
                EmbeddedResource {
                    generation: generation.clone(),
                },
            ));
        }

        let mut cache = self.cache.write().await;
        for (canonical_uri, _) in &discovered {
            if let Some(existing) = cache
                .documents
                .get(canonical_uri)
                .filter(|existing| &existing.source_schema_uri != source_schema_uri.as_ref())
            {
                return Err(crate::Error::InvalidSchemaResources {
                    schema_uri: source_schema_uri.as_ref().clone(),
                    reason: format!(
                        "duplicate schema resource URI {canonical_uri}; already loaded from {}",
                        existing.source_schema_uri
                    ),
                });
            }
        }
        let registry = &mut cache.resources;
        if registry
            .by_source
            .get(source_schema_uri.as_ref())
            .is_some_and(|installed| installed.revision > generation.revision)
        {
            return Ok(false);
        }
        for (canonical_uri, _) in &discovered {
            if let Some(existing) = registry.by_uri.get(canonical_uri).filter(|existing| {
                existing.generation.source_schema_uri.as_ref() != source_schema_uri.as_ref()
            }) {
                return Err(crate::Error::InvalidSchemaResources {
                    schema_uri: source_schema_uri.as_ref().clone(),
                    reason: format!(
                        "duplicate schema resource URI {canonical_uri}; already loaded from {}",
                        existing.generation.source_schema_uri
                    ),
                });
            }
        }
        registry.by_uri.retain(|_, resource| {
            resource.generation.source_schema_uri.as_ref() != source_schema_uri.as_ref()
        });
        registry.by_uri.extend(discovered);
        registry
            .by_source
            .insert(source_schema_uri.as_ref().clone(), generation.clone());
        Ok(true)
    }

    pub(crate) async fn effective_base_uri(
        &self,
        schema_uri: &SchemaUri,
        position: tombi_text::Position,
    ) -> Option<SchemaUri> {
        let cache = self.cache.read().await;
        let registry = &cache.resources;
        registry
            .by_uri
            .get(schema_uri)
            .map(|resource| &resource.generation.resource_index)
            .or_else(|| {
                registry
                    .by_source
                    .get(schema_uri)
                    .map(|generation| &generation.resource_index)
            })
            .and_then(|index| index.effective_base(position))
            .cloned()
    }

    pub fn try_get_document_schema<'a: 'b, 'b>(
        &'a self,
        schema_uri: &'a SchemaUri,
    ) -> BoxFuture<'b, Result<Option<Arc<DocumentSchema>>, crate::Error>> {
        async move {
            if RESOLUTION_STACK.try_with(|_| ()).is_err() {
                return RESOLUTION_STACK
                    .scope(
                        RefCell::new(Vec::new()),
                        self.try_get_document_schema(schema_uri),
                    )
                    .await;
            }
            let requested_schema_uri = schema_uri.clone();
            let (schema_uri, fragment) = {
                let mut uri = schema_uri.clone();
                let fragment = uri.fragment().map(ToOwned::to_owned);
                uri.set_fragment(None);
                (uri, fragment)
            };

            let (embedded_resource, cached_document_schema) = {
                let cache = self.cache.read().await;
                (
                    cache.resources.by_uri.get(&schema_uri).cloned(),
                    cache.documents.get(&schema_uri).cloned(),
                )
            };
            let document_schema = if let Some(resource) = embedded_resource {
                self.document_schema_from_generation(&schema_uri, resource.generation)
                    .await?
            } else if let Some(cached_document_schema) = cached_document_schema {
                let cache_version =
                    schema_cache_version(&cached_document_schema.source_schema_uri).await;
                if cached_document_schema.version == cache_version {
                    match cached_document_schema.document_schema {
                        Ok(document_schema) => Some(document_schema),
                        Err(err) => return Err(err),
                    }
                } else {
                    let source_schema_uri = cached_document_schema.source_schema_uri;
                    match self
                        .fetch_stable_document_schema(&source_schema_uri)
                        .await
                        .transpose()
                    {
                        Some(source_document_schema) => {
                            let (source_document_schema, cache_version) = source_document_schema?;
                            self.replace_cached_source_document(
                                source_schema_uri.clone(),
                                cache_version,
                                source_document_schema.clone(),
                            )
                            .await?;
                            let document_schema = if schema_uri == source_schema_uri {
                                source_document_schema
                            } else {
                                let Some(document_schema) =
                                    self.fetch_document_schema(&schema_uri).await?
                                else {
                                    return Ok(None);
                                };
                                document_schema
                            };
                            self.cache_document_schema(
                                schema_uri.clone(),
                                source_schema_uri,
                                cache_version,
                                document_schema.clone(),
                            )
                            .await?;
                            Some(document_schema)
                        }
                        None => None,
                    }
                }
            } else {
                match self
                    .fetch_stable_document_schema(&schema_uri)
                    .await
                    .transpose()
                {
                    Some(document_schema) => {
                        let (document_schema, cache_version) = document_schema?;
                        let source_schema_uri = document_schema.schema_uri.clone();
                        self.cache_document_schema(
                            schema_uri.clone(),
                            source_schema_uri,
                            cache_version,
                            document_schema.clone(),
                        )
                        .await?;
                        Some(document_schema)
                    }
                    None => None,
                }
            };

            let Some(document_schema) = document_schema else {
                return Ok(None);
            };

            // If no fragment, return the base document schema as-is
            let Some(fragment) = fragment else {
                return Ok(Some(document_schema));
            };

            let fragment_reference = format!("#{fragment}");

            // Handle JSON Pointer fragments (e.g., "#/definitions/TableValue")
            if fragment_reference == "#" || fragment_reference.starts_with("#/") {
                let Some(schema_value) = self.fetch_schema_value(&schema_uri).await? else {
                    return Ok(None);
                };

                let Some(fragment_schema_view) = resolve_json_pointer(
                    &schema_value,
                    &fragment_reference,
                    document_schema.string_formats(),
                    document_schema.dialect(),
                )?
                else {
                    return Err(crate::Error::InvalidJsonPointer {
                        pointer: fragment_reference,
                        schema_uri,
                    });
                };

                let mut fragment_document_schema = document_schema.as_ref().clone();
                fragment_document_schema.schema_uri = requested_schema_uri;
                fragment_document_schema.schema_view = Some(Arc::new(fragment_schema_view));
                return Ok(Some(Arc::new(fragment_document_schema)));
            }

            // Handle anchor fragments (e.g., "#anchorName")
            let anchor_schema = {
                let anchors = document_schema.anchors.read().await;
                if let Some(schema) = anchors.get(&fragment_reference).cloned() {
                    Some(schema)
                } else {
                    drop(anchors);
                    let dynamic_anchors = document_schema.dynamic_anchors.read().await;
                    dynamic_anchors.get(&fragment_reference).cloned()
                }
            };

            if let Some(anchor_schema) = anchor_schema
                && let Some(current_schema) = anchor_schema
                    .to_current_schema(
                        Cow::Borrowed(document_schema.base_uri()),
                        Cow::Borrowed(&document_schema.definitions),
                        None,
                        self,
                    )
                    .await?
            {
                let mut fragment_document_schema = document_schema.as_ref().clone();
                fragment_document_schema.schema_uri = requested_schema_uri;
                fragment_document_schema.schema_view = Some(current_schema.schema_view);
                return Ok(Some(Arc::new(fragment_document_schema)));
            }

            Err(crate::Error::InvalidJsonSchemaReference {
                reference: fragment_reference,
                schema_uri,
            })
        }
        .boxed()
    }

    async fn cache_document_schema(
        &self,
        key: SchemaUri,
        source_schema_uri: SchemaUri,
        version: Option<SchemaCacheVersion>,
        document_schema: Arc<DocumentSchema>,
    ) -> Result<(), crate::Error> {
        let cached = CachedDocumentSchema {
            version,
            source_schema_uri: source_schema_uri.clone(),
            document_schema: Ok(document_schema.clone()),
        };
        let mut cache = self.cache.write().await;
        if let Some(generation) = document_schema.owner_generation.as_ref()
            && !cache
                .resources
                .by_source
                .get(&source_schema_uri)
                .is_some_and(|installed| Arc::ptr_eq(installed, generation))
        {
            return Ok(());
        }
        for uri in std::iter::once(&key).chain(document_schema.id.as_ref()) {
            if let Some(resource) = cache.resources.by_uri.get(uri).filter(|resource| {
                resource.generation.source_schema_uri.as_ref() != &source_schema_uri
            }) {
                return Err(crate::Error::InvalidSchemaResources {
                    schema_uri: source_schema_uri.clone(),
                    reason: format!(
                        "duplicate schema resource URI {uri}; already loaded from {}",
                        resource.generation.source_schema_uri
                    ),
                });
            }
        }
        let schemas = &mut cache.documents;
        schemas.insert(key, cached.clone());
        if let Some(id) = &document_schema.id {
            schemas.insert(id.clone(), cached);
        }
        Ok(())
    }

    /// Atomically replaces every alias belonging to one physical document.
    /// This prevents removed `$id` values from surviving a successful reload.
    async fn replace_cached_source_document(
        &self,
        source_schema_uri: SchemaUri,
        version: Option<SchemaCacheVersion>,
        document_schema: Arc<DocumentSchema>,
    ) -> Result<(), crate::Error> {
        let cached = CachedDocumentSchema {
            version,
            source_schema_uri: source_schema_uri.clone(),
            document_schema: Ok(document_schema.clone()),
        };
        let mut cache = self.cache.write().await;
        if let Some(generation) = document_schema.owner_generation.as_ref()
            && !cache
                .resources
                .by_source
                .get(&source_schema_uri)
                .is_some_and(|installed| Arc::ptr_eq(installed, generation))
        {
            return Ok(());
        }
        for uri in std::iter::once(&source_schema_uri).chain(document_schema.id.as_ref()) {
            if let Some(resource) = cache.resources.by_uri.get(uri).filter(|resource| {
                resource.generation.source_schema_uri.as_ref() != &source_schema_uri
            }) {
                return Err(crate::Error::InvalidSchemaResources {
                    schema_uri: source_schema_uri.clone(),
                    reason: format!(
                        "duplicate schema resource URI {uri}; already loaded from {}",
                        resource.generation.source_schema_uri
                    ),
                });
            }
        }
        let schemas = &mut cache.documents;
        schemas.retain(|_, existing| existing.source_schema_uri != source_schema_uri);
        schemas.insert(source_schema_uri, cached.clone());
        if let Some(id) = &document_schema.id {
            schemas.insert(id.clone(), cached);
        }
        Ok(())
    }

    #[inline]
    #[cfg(feature = "ast-syntax")]
    async fn try_get_source_schema_from_remote_url(
        &self,
        schema_uri: &SchemaUri,
        source_path: Option<&std::path::Path>,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        let source_schema = if let Some(source_path) = source_path {
            self.resolve_source_schema_from_path(source_path)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        let (
            root_schema,
            sub_schema_link_map,
            toml_version,
            deprecated_lint_level,
            schema_format_rules,
            schema_lint_rules,
            schema_overrides,
            strict,
        ) = if let Some(source_schema) = source_schema {
            let toml_version = source_schema.toml_version();
            let strict = source_schema
                .root_schema
                .as_ref()
                .and_then(|schema| schema.strict);
            (
                source_schema.root_schema,
                source_schema.sub_schema_link_map,
                toml_version,
                source_schema.deprecated_lint_level,
                source_schema.schema_format_rules,
                source_schema.schema_lint_rules,
                source_schema.schema_overrides,
                strict,
            )
        } else {
            (
                None,
                Default::default(),
                None,
                None,
                Default::default(),
                Default::default(),
                Default::default(),
                None,
            )
        };

        let mut root_schema = self
            .try_get_document_schema(schema_uri)
            .await?
            .or(root_schema);
        if let Some(root_schema) = &mut root_schema {
            Arc::make_mut(root_schema).strict = strict;
        }
        let source_schema = SourceSchema::new(
            root_schema,
            sub_schema_link_map,
            toml_version,
            deprecated_lint_level,
            schema_format_rules,
            schema_lint_rules,
            schema_overrides,
        );
        Ok(Some(source_schema))
    }

    #[cfg(feature = "ast-syntax")]
    // Preserve the existing tuple error API without boxing every failure.
    #[allow(clippy::result_large_err)]
    pub async fn resolve_source_schema_from_ast(
        &self,
        root: &tombi_ast_syntax::Root,
        source_uri_or_path: Option<Either<&tombi_uri::Uri, &std::path::Path>>,
    ) -> Result<Option<SourceSchema>, (crate::Error, tombi_text::Range)> {
        let source_path = match source_uri_or_path {
            Some(Either::Left(url)) => match url.scheme() {
                "file" => tombi_uri::Uri::to_file_path(url).ok(),
                _ => None,
            },
            Some(Either::Right(path)) => {
                Some(std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()))
            }
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
                .try_get_source_schema_from_remote_url(&schema_uri, source_path.as_deref())
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
        let canonicalized_source_path = canonicalize_path_for_matching(source_path);

        // Get the base directory for relative path conversion
        let base_dir_path = self.base_dir_path.read().await;

        // Determine the path to use for pattern matching without per-call filesystem I/O
        let path_for_matching = base_dir_path
            .as_deref()
            .and_then(|base_dir_path| {
                canonicalized_source_path
                    .strip_prefix(base_dir_path)
                    .ok()
                    .map(|relative_source_path| relative_source_path.to_path_buf())
            })
            .unwrap_or_else(|| canonicalized_source_path.clone());

        let schemas = self.schemas.read().await;
        let matching_schemas = schemas
            .iter()
            .filter(|schema| schema.matches(&path_for_matching, &canonicalized_source_path))
            .collect_vec();

        let mut source_schema: Option<SourceSchema> = None;
        let mut sub_schemas_inheriting_strict = Vec::new();
        for matching_schema in matching_schemas {
            // Skip if the same schema (by URL and sub_root_accessors) is already loaded in source_schema
            let already_loaded = match &matching_schema.sub_root_accessors {
                Some(sub_root_accessors) => source_schema.as_ref().is_some_and(|source_schema| {
                    source_schema
                        .sub_schema_link_map
                        .contains_key(sub_root_accessors)
                }),
                None => source_schema
                    .as_ref()
                    .is_some_and(|source_schema| source_schema.root_schema.is_some()),
            };
            if already_loaded {
                continue;
            }
            match self
                .try_get_document_schema(&matching_schema.schema_uri)
                .await
            {
                Ok(Some(document_schema)) => match &matching_schema.sub_root_accessors {
                    Some(sub_root_accessors) => match source_schema {
                        Some(ref mut source_schema) => {
                            if !source_schema
                                .sub_schema_link_map
                                .contains_key(sub_root_accessors)
                            {
                                let schema_uri_key =
                                    Self::normalize_schema_uri_key(&document_schema.schema_uri);
                                if matching_schema.strict.is_none() {
                                    sub_schemas_inheriting_strict.push(sub_root_accessors.clone());
                                }
                                source_schema.sub_schema_link_map.insert(
                                    sub_root_accessors.clone(),
                                    SubSchemaLink {
                                        schema_uri: document_schema.schema_uri.clone(),
                                        strict: matching_schema
                                            .strict
                                            .or_else(|| self.strict())
                                            .unwrap_or_default()
                                            .value(),
                                    },
                                );
                                if let Some(format_rules) = &matching_schema.format_rules {
                                    source_schema
                                        .schema_format_rules
                                        .insert(schema_uri_key.clone(), format_rules.clone());
                                }
                                if let Some(lint_rules) = &matching_schema.lint_rules {
                                    source_schema
                                        .schema_lint_rules
                                        .insert(schema_uri_key.clone(), lint_rules.clone());
                                }
                                source_schema
                                    .schema_overrides
                                    .insert(schema_uri_key, matching_schema.overrides.clone());
                            }
                        }
                        None => {
                            let schema_uri_key =
                                Self::normalize_schema_uri_key(&document_schema.schema_uri);
                            if matching_schema.strict.is_none() {
                                sub_schemas_inheriting_strict.push(sub_root_accessors.clone());
                            }
                            let mut sub_schema_link_map = SubSchemaLinkMap::default();
                            sub_schema_link_map.insert(
                                sub_root_accessors.clone(),
                                SubSchemaLink {
                                    schema_uri: document_schema.schema_uri.clone(),
                                    strict: matching_schema
                                        .strict
                                        .or_else(|| self.strict())
                                        .unwrap_or_default()
                                        .value(),
                                },
                            );
                            let mut schema_format_rules = crate::SchemaFormatRulesMap::default();
                            if let Some(format_rules) = &matching_schema.format_rules {
                                schema_format_rules
                                    .insert(schema_uri_key.clone(), format_rules.clone());
                            }
                            let mut schema_lint_rules = crate::SchemaLintRulesMap::default();
                            if let Some(lint_rules) = &matching_schema.lint_rules {
                                schema_lint_rules
                                    .insert(schema_uri_key.clone(), lint_rules.clone());
                            }
                            let mut schema_overrides = crate::SchemaOverridesMap::default();
                            schema_overrides
                                .insert(schema_uri_key, matching_schema.overrides.clone());
                            let new_source = SourceSchema::new(
                                None,
                                sub_schema_link_map,
                                matching_schema.toml_version,
                                matching_schema.deprecated_lint_level,
                                schema_format_rules,
                                schema_lint_rules,
                                schema_overrides,
                            );
                            source_schema = Some(new_source);
                        }
                    },
                    None => match source_schema {
                        Some(ref mut existing) => {
                            if existing.root_schema.is_none() {
                                let schema_uri_key =
                                    Self::normalize_schema_uri_key(&document_schema.schema_uri);
                                let toml_version =
                                    existing.toml_version().or(matching_schema.toml_version);
                                let sub_schema_link_map =
                                    std::mem::take(&mut existing.sub_schema_link_map);
                                let mut schema_format_rules =
                                    std::mem::take(&mut existing.schema_format_rules);
                                let mut schema_lint_rules =
                                    std::mem::take(&mut existing.schema_lint_rules);
                                let mut schema_overrides =
                                    std::mem::take(&mut existing.schema_overrides);
                                if let Some(format_rules) = &matching_schema.format_rules {
                                    schema_format_rules
                                        .insert(schema_uri_key.clone(), format_rules.clone());
                                }
                                if let Some(lint_rules) = &matching_schema.lint_rules {
                                    schema_lint_rules
                                        .insert(schema_uri_key.clone(), lint_rules.clone());
                                }
                                schema_overrides
                                    .insert(schema_uri_key, matching_schema.overrides.clone());
                                let mut document_schema = document_schema;
                                Arc::make_mut(&mut document_schema).strict = matching_schema.strict;
                                *existing = SourceSchema::new(
                                    Some(document_schema),
                                    sub_schema_link_map,
                                    toml_version,
                                    matching_schema.deprecated_lint_level,
                                    schema_format_rules,
                                    schema_lint_rules,
                                    schema_overrides,
                                );
                            }
                        }
                        None => {
                            let schema_uri_key =
                                Self::normalize_schema_uri_key(&document_schema.schema_uri);
                            let mut schema_format_rules = crate::SchemaFormatRulesMap::default();
                            if let Some(format_rules) = &matching_schema.format_rules {
                                schema_format_rules
                                    .insert(schema_uri_key.clone(), format_rules.clone());
                            }
                            let mut schema_lint_rules = crate::SchemaLintRulesMap::default();
                            if let Some(lint_rules) = &matching_schema.lint_rules {
                                schema_lint_rules
                                    .insert(schema_uri_key.clone(), lint_rules.clone());
                            }
                            let mut schema_overrides = crate::SchemaOverridesMap::default();
                            schema_overrides
                                .insert(schema_uri_key, matching_schema.overrides.clone());
                            let mut document_schema = document_schema;
                            Arc::make_mut(&mut document_schema).strict = matching_schema.strict;
                            let new_source = SourceSchema::new(
                                Some(document_schema),
                                Default::default(),
                                matching_schema.toml_version,
                                matching_schema.deprecated_lint_level,
                                schema_format_rules,
                                schema_lint_rules,
                                schema_overrides,
                            );
                            source_schema = Some(new_source);
                        }
                    },
                },
                Ok(None) => {
                    log::warn!(
                        "failed to find document schema: {}",
                        matching_schema.schema_uri
                    );
                }
                Err(err) => {
                    log::warn!(
                        "failed to get document schema for {url}: {err}",
                        url = matching_schema.schema_uri,
                    );
                }
            }
        }

        if let Some(source_schema) = &mut source_schema {
            let inherited_strict = source_schema
                .root_schema
                .as_ref()
                .and_then(|schema| schema.strict)
                .or_else(|| self.strict())
                .unwrap_or_default()
                .value();
            for root_accessors in sub_schemas_inheriting_strict {
                if let Some(link) = source_schema.sub_schema_link_map.get_mut(&root_accessors) {
                    link.strict = inherited_strict;
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
                    log::trace!("find root schema from {}", root_schema.schema_uri);
                }
                for (accessors, link) in &source_schema.sub_schema_link_map {
                    log::trace!(
                        "find sub schema {:?} from {}",
                        PatternAccessors::from(accessors.clone()),
                        link.schema_uri
                    );
                }
            }
        })
    }

    pub async fn associate_schema(
        &self,
        schema_uri: SchemaUri,
        include: Vec<String>,
        options: &AssociateSchemaOptions,
    ) {
        let include = include
            .into_iter()
            .map(|pattern| canonicalize_file_match_pattern(&pattern))
            .collect();

        let new_schema = crate::Schema {
            title: options.title.clone(),
            description: options.description.clone(),
            deprecated_lint_level: None,
            format_rules: None,
            lint_rules: None,
            overrides: Default::default(),
            strict: None,
            schema_uri,
            catalog_uri: None,
            include,
            exclude: None,
            toml_version: options.toml_version,
            sub_root_accessors: None,
        };

        let mut schemas = self.schemas.write().await;
        if options.force {
            // Insert at the beginning to force precedence
            schemas.insert(0, StoredSchema::new(new_schema));
        } else {
            // Append at the end
            schemas.push(StoredSchema::new(new_schema));
        }
    }

    pub async fn list_schemas(&self) -> Vec<crate::Schema> {
        self.schemas
            .read()
            .await
            .iter()
            .map(|schema| schema.schema.clone())
            .collect()
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn schema_cache_version(schema_uri: &SchemaUri) -> Option<SchemaCacheVersion> {
    let path = match schema_uri.scheme() {
        "file" => tombi_uri::Uri::to_file_path(schema_uri).ok(),
        "http" | "https" => get_cache_file_path(schema_uri).await,
        _ => None,
    }?;

    let metadata = tokio::fs::metadata(path).await.ok()?;
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(std::time::UNIX_EPOCH).ok()?;

    Some(SchemaCacheVersion {
        modified_at_nanos: duration
            .as_secs()
            .saturating_mul(1_000_000_000)
            .saturating_add(u64::from(duration.subsec_nanos())),
        len: metadata.len(),
    })
}

#[cfg(target_arch = "wasm32")]
async fn schema_cache_version(_schema_uri: &SchemaUri) -> Option<SchemaCacheVersion> {
    None
}

fn compile_schema_patterns(patterns: &[String]) -> Vec<glob::Pattern> {
    patterns
        .iter()
        .filter_map(|pattern| glob::Pattern::new(&glob_pattern_for_file_match(pattern)).ok())
        .collect()
}

#[cfg(test)]
fn matches_schema_patterns(
    include: &[String],
    exclude: Option<&[String]>,
    path_for_matching: &std::path::Path,
    absolute_source_path: &std::path::Path,
) -> bool {
    SchemaPatterns::new(include, exclude).matches(path_for_matching, absolute_source_path)
}

fn glob_pattern_for_file_match(pattern: &str) -> String {
    if pattern.contains('*') || std::path::Path::new(pattern).is_absolute() {
        pattern.to_string()
    } else {
        format!("**/{pattern}")
    }
}

fn canonicalize_file_match_pattern(pattern: &str) -> String {
    let path = std::path::Path::new(pattern);
    if pattern.contains('*') || !path.is_absolute() {
        pattern.to_string()
    } else {
        canonicalize_path_for_matching(path)
            .to_string_lossy()
            .into_owned()
    }
}

async fn load_catalog_from_cache_ignoring_ttl(
    tagalog_uri: &CatalogUri,
    catalog_cache_path: Option<&std::path::Path>,
    cache_options: Option<tombi_cache::Options>,
) -> Result<Option<JsonCatalog>, crate::Error> {
    if let Some(catalog_cache_path) = catalog_cache_path {
        let mut owned_cache_options = cache_options.unwrap_or_default();
        owned_cache_options.cache_ttl = None;
        if let Ok(Some(catalog)) =
            load_catalog_from_cache(tagalog_uri, catalog_cache_path, Some(&owned_cache_options))
                .await
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
        log::debug!("load catalog from cache: {}", tagalog_uri);

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
        let mut owned_cache_options = cache_options.unwrap_or_default();
        owned_cache_options.cache_ttl = None;
        if let Ok(Some(schema_value)) =
            load_json_schema_from_cache(schema_uri, schema_cache_path, Some(&owned_cache_options))
                .await
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
        log::trace!("load schema from cache: {}", schema_uri);

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

fn canonicalize_path_for_matching(path: &std::path::Path) -> std::path::PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .ok()
                .map(|current_dir| current_dir.join(path))
                .unwrap_or_else(|| path.to_path_buf())
        }
    })
}

fn schema_overrides(schema: &tombi_config::SchemaItem) -> crate::SchemaOverrides {
    let mut overrides = crate::SchemaOverrides::default();

    for override_item in schema.overrides().into_iter().flatten() {
        let format_rules = override_item
            .format
            .as_ref()
            .and_then(|format| format.rules.as_ref());
        let lint_rules = override_item
            .lint
            .as_ref()
            .and_then(|lint| lint.rules.as_ref());
        let targets = override_item
            .targets
            .iter()
            .filter_map(|target| parse_override_target(target))
            .collect_vec();

        if let Some(level) = lint_rules.and_then(|r| r.deprecated) {
            overrides.deprecated.extend(
                targets
                    .iter()
                    .cloned()
                    .map(|target| crate::DeprecatedOverride { target, level }),
            );
        }

        if let Some(rule) = format_rules.and_then(|r| r.array_values_order.as_ref()) {
            let disabled = !rule.enabled().unwrap_or_default().value();
            let order = rule.order();
            overrides
                .array_values_order
                .extend(
                    targets
                        .iter()
                        .cloned()
                        .map(|target| crate::ArrayOrderOverride {
                            target,
                            disabled,
                            order,
                        }),
                );
        }

        if let Some(rule) = format_rules.and_then(|r| r.table_keys_order.as_ref()) {
            let disabled = !rule.enabled().unwrap_or_default().value();
            let order = rule.order();
            overrides
                .table_keys_order
                .extend(
                    targets
                        .iter()
                        .cloned()
                        .map(|target| crate::TableOrderOverride {
                            target,
                            disabled,
                            order,
                        }),
                );
        }
    }

    overrides
}

fn parse_override_target(target: &str) -> Option<Vec<PatternAccessor>> {
    if target.is_empty() {
        Some(Vec::new())
    } else {
        PatternAccessor::parse(target)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        str::FromStr,
        time::Duration,
    };

    use super::{
        SchemaGeneration, SchemaStore, load_catalog_from_cache_ignoring_ttl,
        load_json_schema_from_cache_ignoring_ttl, matches_schema_patterns,
    };
    use crate::{CatalogUri, ResourceIndex, SchemaAccessor, SchemaView};
    use tombi_future::Boxable;
    use tombi_uri::SchemaUri;

    fn temp_cache_path(test_name: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tombi-schema-store-{test_name}-{unique}.json"))
    }

    fn bump_modified(path: &Path) {
        let file = fs::File::options().write(true).open(path).unwrap();
        let modified = file.metadata().unwrap().modified().unwrap() + Duration::from_secs(1);
        file.set_modified(modified).unwrap();
    }

    #[derive(Debug)]
    struct CountingHttpClient {
        requests: AtomicUsize,
        response: &'static str,
    }

    impl crate::http_client::HttpClient for CountingHttpClient {
        fn get_bytes<'a>(
            &'a self,
            _url: &'a str,
        ) -> crate::http_client::HttpFuture<'a, Result<bytes::Bytes, crate::http_client::FetchError>>
        {
            self.requests.fetch_add(1, Ordering::Relaxed);
            async move { Ok(bytes::Bytes::from_static(self.response.as_bytes())) }.boxed()
        }
    }

    #[test]
    fn schema_include_matches_user_config_path_via_absolute_suffix() {
        assert!(matches_schema_patterns(
            &[String::from("tombi/config.toml")],
            None,
            Path::new("config.toml"),
            Path::new("/Users/test/.config/tombi/config.toml"),
        ));
    }

    #[test]
    fn schema_include_does_not_match_unrelated_config_toml() {
        assert!(!matches_schema_patterns(
            &[String::from("tombi/config.toml")],
            None,
            Path::new("config.toml"),
            Path::new("/Users/test/project/config.toml"),
        ));
    }

    #[test]
    fn schema_exclude_blocks_matching_path() {
        assert!(!matches_schema_patterns(
            &[String::from("**/*.toml")],
            Some(&[String::from("vendor/**/*.toml")]),
            Path::new("vendor/blocked.toml"),
            Path::new("/Users/test/project/vendor/blocked.toml"),
        ));
    }

    #[test]
    fn schema_include_matches_absolute_posix_path() {
        let absolute_path = Path::new("/Users/test/project/selected-schema.toml");
        let include = [absolute_path.to_string_lossy().into_owned()];

        assert!(matches_schema_patterns(
            &include,
            None,
            absolute_path,
            absolute_path,
        ));
    }

    #[cfg(windows)]
    #[test]
    fn schema_include_matches_absolute_windows_path() {
        let absolute_path = Path::new(r"C:\Users\test\project\selected-schema.toml");
        let include = [absolute_path.to_string_lossy().into_owned()];

        assert!(matches_schema_patterns(
            &include,
            None,
            absolute_path,
            absolute_path,
        ));
    }

    #[tokio::test]
    async fn fragment_pointer_resolves_boolean_schema() {
        let schema_path = std::env::temp_dir().join(format!(
            "tombi_fragment_boolean_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(
            &schema_path,
            r#"{
                "$defs": {
                    "allowAll": true
                }
            }"#,
        )
        .unwrap();

        let schema_uri = SchemaUri::from_str(&format!(
            "{}#/$defs/allowAll",
            SchemaUri::from_file_path(&schema_path).unwrap()
        ))
        .unwrap();
        let schema_store = SchemaStore::new();

        let document_schema = schema_store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();

        std::assert_matches!(
            document_schema.schema_view.as_deref(),
            Some(SchemaView::Anything(_))
        );

        let _ = std::fs::remove_file(schema_path);
    }

    #[tokio::test]
    async fn fragment_anchor_resolves_dynamic_anchor() {
        let schema_path = std::env::temp_dir().join(format!(
            "tombi_fragment_dynamic_anchor_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(
            &schema_path,
            r#"{
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$defs": {
                    "name": {
                        "$dynamicAnchor": "nameSchema",
                        "type": "string"
                    }
                }
            }"#,
        )
        .unwrap();

        let schema_uri = SchemaUri::from_str(&format!(
            "{}#nameSchema",
            SchemaUri::from_file_path(&schema_path).unwrap()
        ))
        .unwrap();
        let schema_store = SchemaStore::new();

        let document_schema = schema_store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();

        std::assert_matches!(
            document_schema.schema_view.as_deref(),
            Some(SchemaView::String(_))
        );

        let _ = std::fs::remove_file(schema_path);
    }

    #[tokio::test]
    async fn reloads_cached_file_schema_when_file_changes() {
        let schema_path = std::env::temp_dir().join(format!(
            "tombi_reload_schema_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let schema_uri = SchemaUri::from_file_path(&schema_path).unwrap();
        let schema_store = SchemaStore::new();

        std::fs::write(&schema_path, r#"{"type":"string"}"#).unwrap();

        let document_schema = schema_store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();

        std::assert_matches!(
            document_schema.schema_view.as_deref(),
            Some(SchemaView::String(_))
        );

        std::fs::write(&schema_path, r#"{"type":"integer"}"#).unwrap();
        bump_modified(&schema_path);

        let document_schema = schema_store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();

        std::assert_matches!(
            document_schema.schema_view.as_deref(),
            Some(SchemaView::Integer(_))
        );

        let _ = std::fs::remove_file(schema_path);
    }

    #[tokio::test]
    async fn retained_document_resolves_with_its_original_generation() {
        let schema_path = std::env::temp_dir().join(format!(
            "tombi_reload_compound_schema_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let schema_uri = SchemaUri::from_file_path(&schema_path).unwrap();
        let schema_store = SchemaStore::new();
        let schema = |value_type: &str| {
            format!(
                r#"{{
                    "$schema":"https://json-schema.org/draft/2020-12/schema",
                    "$id":"https://example.com/root",
                    "type":"object",
                    "properties":{{"value":{{"$ref":"child"}}}},
                    "$defs":{{"child":{{"$id":"https://example.com/child","type":"{value_type}"}}}}
                }}"#
            )
        };

        std::fs::write(&schema_path, schema("string")).unwrap();
        let old_document = schema_store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();

        std::fs::write(&schema_path, schema("integer")).unwrap();
        bump_modified(&schema_path);
        let new_document = schema_store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();

        async fn value_schema(
            document: &crate::DocumentSchema,
            store: &SchemaStore,
        ) -> crate::CurrentSchema<'static> {
            let SchemaView::Table(table) = document.schema_view.as_deref().unwrap() else {
                panic!("root schema must be an object")
            };
            table
                .resolve_property_schema(
                    &SchemaAccessor::Key("value".to_owned()),
                    Cow::Borrowed(&document.schema_uri),
                    Cow::Borrowed(&document.definitions),
                    document.strict,
                    store,
                )
                .await
                .unwrap()
                .unwrap()
        }

        let old_value = value_schema(&old_document, &schema_store).await;
        let new_value = value_schema(&new_document, &schema_store).await;
        assert!(matches!(
            old_value.schema_view.as_ref(),
            SchemaView::String(_)
        ));
        assert!(matches!(
            new_value.schema_view.as_ref(),
            SchemaView::Integer(_)
        ));

        let _ = std::fs::remove_file(schema_path);
    }

    #[tokio::test]
    async fn retained_generation_miss_does_not_see_new_embedded_resource() {
        let schema_path = temp_cache_path("generation-negative-lookup");
        let external_path = schema_path.with_extension("child.json");
        let external_name = external_path.file_name().unwrap().to_string_lossy();
        let schema_uri = SchemaUri::from_file_path(&schema_path).unwrap();
        let schema = |embedded_type: Option<&str>| {
            let defs = embedded_type.map_or_else(String::new, |value_type| {
                format!(
                    r#", "$defs":{{"child":{{"$id":"{external_name}","type":"{value_type}","$defs":{{"nested":{{"$id":"nested","type":"{value_type}"}}}}}}}}"#
                )
            });
            format!(
                r#"{{
                    "$schema":"https://json-schema.org/draft/2020-12/schema",
                    "type":"object",
                    "properties":{{"value":{{"$ref":"{external_name}"}}}}
                    {defs}
                }}"#
            )
        };
        std::fs::write(
            &external_path,
            r#"{"type":"string","$defs":{"nested":{"$id":"nested","type":"string"}}}"#,
        )
        .unwrap();
        std::fs::write(&schema_path, schema(None)).unwrap();
        let store = SchemaStore::new();
        let old_document = store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();

        std::fs::write(&schema_path, schema(Some("integer"))).unwrap();
        bump_modified(&schema_path);
        let new_document = store
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();

        async fn value_schema(
            document: &crate::DocumentSchema,
            store: &SchemaStore,
        ) -> crate::CurrentSchema<'static> {
            let SchemaView::Table(table) = document.schema_view.as_deref().unwrap() else {
                panic!("root schema must be an object")
            };
            table
                .resolve_property_schema(
                    &SchemaAccessor::Key("value".to_owned()),
                    Cow::Borrowed(&document.schema_uri),
                    Cow::Borrowed(&document.definitions),
                    document.strict,
                    store,
                )
                .await
                .unwrap()
                .unwrap()
        }

        assert!(matches!(
            value_schema(&old_document, &store)
                .await
                .schema_view
                .as_ref(),
            SchemaView::String(_)
        ));
        assert!(matches!(
            value_schema(&new_document, &store)
                .await
                .schema_view
                .as_ref(),
            SchemaView::Integer(_)
        ));
        let nested_uri =
            SchemaUri::from_file_path(external_path.parent().unwrap().join("nested")).unwrap();
        let new_generation = new_document.definitions.generation().unwrap();
        let cache = store.cache.read().await;
        let external_uri = SchemaUri::from_file_path(&external_path).unwrap();
        assert!(!cache.resources.by_source.contains_key(&external_uri));
        assert!(
            cache
                .resources
                .by_uri
                .get(&nested_uri)
                .is_some_and(|resource| { Arc::ptr_eq(&resource.generation, new_generation) })
        );
        drop(cache);

        let _ = std::fs::remove_file(schema_path);
        let _ = std::fs::remove_file(external_path);
    }

    #[tokio::test]
    async fn embedded_resource_falls_back_to_external_file() {
        let schema_path = temp_cache_path("compound-external-fallback");
        let external_path = schema_path.with_extension("external.json");
        let external_name = external_path.file_name().unwrap().to_string_lossy();
        let schema_uri = SchemaUri::from_file_path(&schema_path).unwrap();
        std::fs::write(&external_path, r#"{"type":"string"}"#).unwrap();
        std::fs::write(
            &schema_path,
            format!(
                r#"{{
                    "$schema":"https://json-schema.org/draft/2020-12/schema",
                    "$ref":"embedded",
                    "$defs":{{"embedded":{{"$id":"embedded","$ref":"{external_name}"}}}}
                }}"#
            ),
        )
        .unwrap();

        let document_schema = SchemaStore::new()
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(
            document_schema.schema_view.as_deref(),
            Some(SchemaView::String(_))
        ));

        let _ = std::fs::remove_file(schema_path);
        let _ = std::fs::remove_file(external_path);
    }

    #[tokio::test]
    async fn duplicate_embedded_resource_ids_are_rejected_by_store() {
        let schema_path = temp_cache_path("duplicate-compound-resource");
        let schema_uri = SchemaUri::from_file_path(&schema_path).unwrap();
        std::fs::write(
            &schema_path,
            r#"{
                "$schema":"https://json-schema.org/draft/2020-12/schema",
                "$defs":{
                    "one":{"$id":"duplicate","type":"string"},
                    "two":{"$id":"duplicate","type":"integer"}
                }
            }"#,
        )
        .unwrap();

        let error = SchemaStore::new()
            .try_get_document_schema(&schema_uri)
            .await
            .unwrap_err();

        assert!(matches!(error, crate::Error::InvalidSchemaResources { .. }));
        assert!(error.to_string().contains("duplicate schema resource URI"));

        let _ = std::fs::remove_file(schema_path);
    }

    #[tokio::test]
    async fn preloaded_canonical_resource_resolves_across_documents() {
        let target_path = temp_cache_path("preloaded-canonical-target");
        let source_path = temp_cache_path("preloaded-canonical-source");
        let target_uri = SchemaUri::from_file_path(&target_path).unwrap();
        let source_uri = SchemaUri::from_file_path(&source_path).unwrap();
        std::fs::write(
            &target_path,
            r#"{"$id":"shoko://example/B","type":"string"}"#,
        )
        .unwrap();
        std::fs::write(&source_path, r#"{"$ref":"shoko://example/B"}"#).unwrap();

        let store = SchemaStore::new();
        let target = store
            .try_get_document_schema(&target_uri)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            target
                .as_current_schema()
                .unwrap()
                .source_schema_uri()
                .as_ref(),
            &target_uri
        );
        let source = store
            .try_get_document_schema(&source_uri)
            .await
            .unwrap()
            .unwrap();

        assert!(matches!(
            source.schema_view.as_deref(),
            Some(SchemaView::String(_))
        ));
        assert_eq!(
            source
                .as_current_schema()
                .unwrap()
                .source_schema_uri()
                .as_ref(),
            &target_uri
        );

        let _ = std::fs::remove_file(target_path);
        let _ = std::fs::remove_file(source_path);
    }

    #[tokio::test]
    async fn external_json_pointer_reuses_the_fetched_generation() {
        let source_path = temp_cache_path("external-pointer-generation");
        let source_uri = SchemaUri::from_file_path(&source_path).unwrap();
        let external_url = format!(
            "https://example.invalid/{}.json#/$defs/value",
            source_path.file_stem().unwrap().to_string_lossy()
        );
        std::fs::write(
            &source_path,
            format!(
                r#"{{
                    "$defs":{{
                        "useExternal":{{"$ref":"{external_url}"}}
                    }}
                }}"#
            ),
        )
        .unwrap();
        let client = Arc::new(CountingHttpClient {
            requests: AtomicUsize::new(0),
            response: r#"{
                "$defs": {
                    "value": {"type":"string"}
                }
            }"#,
        });
        let store = SchemaStore::new_with_options_and_http_client(
            crate::Options::default(),
            client.clone(),
        );

        let source = store
            .try_get_document_schema(&source_uri)
            .await
            .unwrap()
            .unwrap();

        let definitions = source.definitions.clone();
        let mut referable = {
            let definitions = definitions.read().await;
            definitions.get("#/$defs/useExternal").cloned().unwrap()
        };
        let item = referable
            .resolve(
                Cow::Owned(source.schema_uri.clone()),
                Cow::Owned(definitions.clone()),
                None,
                &store,
            )
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            item.schema_view.as_ref(),
            SchemaView::String(_) | SchemaView::AnyOf(_)
        ));
        let mut external_schema_uri = SchemaUri::from_str(&external_url).unwrap();
        external_schema_uri.set_fragment(None);
        {
            let cache = store.cache.read().await;
            assert!(cache.documents.contains_key(&external_schema_uri));
            assert!(cache.resources.by_source.contains_key(&external_schema_uri));
            assert!(!cache.resources.by_uri.contains_key(&external_schema_uri));
        }
        referable
            .resolve(
                Cow::Owned(source.schema_uri.clone()),
                Cow::Owned(definitions),
                None,
                &store,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(client.requests.load(Ordering::Relaxed), 1);

        let _ = std::fs::remove_file(source_path);
    }

    #[tokio::test]
    async fn standalone_and_embedded_resource_ownership_never_split() {
        let standalone_path = temp_cache_path("standalone-owner");
        let bundle_path = temp_cache_path("embedded-owner");
        let standalone_uri = SchemaUri::from_file_path(&standalone_path).unwrap();
        let bundle_uri = SchemaUri::from_file_path(&bundle_path).unwrap();
        let canonical_uri = "https://example.com/shared-resource";
        std::fs::write(
            &standalone_path,
            format!(r#"{{"$id":"{canonical_uri}","type":"string"}}"#),
        )
        .unwrap();
        std::fs::write(
            &bundle_path,
            format!(r#"{{"$defs":{{"shared":{{"$id":"{canonical_uri}","type":"integer"}}}}}}"#),
        )
        .unwrap();

        let standalone_first = SchemaStore::new();
        standalone_first
            .try_get_document_schema(&standalone_uri)
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            standalone_first.try_get_document_schema(&bundle_uri).await,
            Err(crate::Error::InvalidSchemaResources { .. })
        ));

        let embedded_first = SchemaStore::new();
        embedded_first
            .try_get_document_schema(&bundle_uri)
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            embedded_first
                .try_get_document_schema(&standalone_uri)
                .await,
            Err(crate::Error::InvalidSchemaResources { .. })
        ));

        for _ in 0..16 {
            let concurrent = SchemaStore::new();
            let (standalone, embedded) = tokio::join!(
                concurrent.try_get_document_schema(&standalone_uri),
                concurrent.try_get_document_schema(&bundle_uri),
            );
            assert_ne!(
                standalone.is_ok(),
                embedded.is_ok(),
                "exactly one source must acquire the canonical URI"
            );
        }

        let _ = std::fs::remove_file(standalone_path);
        let _ = std::fs::remove_file(bundle_path);
    }

    #[tokio::test]
    async fn older_generation_cannot_replace_newer_publication() {
        fn generation(
            source: &SchemaUri,
            revision: u64,
            value_type: &str,
        ) -> Arc<SchemaGeneration> {
            let node = Arc::new(
                tombi_json::ValueNode::from_str(&format!(
                    r#"{{"$id":"https://example.com/root","$defs":{{"child":{{"$id":"child","type":"{value_type}"}}}}}}"#
                ))
                .unwrap(),
            );
            Arc::new(SchemaGeneration {
                revision,
                source_schema_uri: Arc::new(source.clone()),
                resource_index: Arc::new(ResourceIndex::build(&node, source, None).unwrap()),
                source_node: node,
                compiled_resources: std::sync::RwLock::new(Default::default()),
            })
        }

        let source = SchemaUri::from_str("file:///tmp/tombi-generation-order.json").unwrap();
        let older = generation(&source, 1, "string");
        let newer = generation(&source, 2, "integer");
        let store = SchemaStore::new();

        assert!(
            store
                .register_embedded_resources(newer.clone())
                .await
                .unwrap()
        );
        assert!(!store.register_embedded_resources(older).await.unwrap());

        let cache = store.cache.read().await;
        assert!(
            cache
                .resources
                .by_source
                .get(&source)
                .is_some_and(|installed| Arc::ptr_eq(installed, &newer))
        );
    }

    #[tokio::test]
    async fn ignores_ttl_for_catalog_cache_without_cache_options() {
        let cache_path = temp_cache_path("catalog-cache-offline-default-options");
        std::fs::write(
            &cache_path,
            r#"{"schemas":[{"name":"test","description":"desc","url":"https://example.invalid/schema.json"}]}"#,
        )
        .unwrap();
        std::fs::File::options()
            .write(true)
            .open(&cache_path)
            .unwrap()
            .set_modified(std::time::SystemTime::now() - Duration::from_secs(60 * 60 * 25))
            .unwrap();

        let catalog = load_catalog_from_cache_ignoring_ttl(
            &CatalogUri::from_str("https://example.invalid/catalog.json").unwrap(),
            Some(&cache_path),
            None,
        )
        .await
        .unwrap();

        assert_eq!(catalog.unwrap().schemas.len(), 1);

        let _ = std::fs::remove_file(cache_path);
    }

    #[tokio::test]
    async fn ignores_ttl_for_schema_cache_without_cache_options() {
        let cache_path = temp_cache_path("schema-cache-offline-default-options");
        std::fs::write(&cache_path, r#"{"type":"string"}"#).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&cache_path)
            .unwrap()
            .set_modified(std::time::SystemTime::now() - Duration::from_secs(60 * 60 * 25))
            .unwrap();

        let schema = load_json_schema_from_cache_ignoring_ttl(
            &SchemaUri::from_str("https://example.invalid/schema.json").unwrap(),
            Some(&cache_path),
            None,
        )
        .await
        .unwrap();

        assert!(schema.is_some());

        let _ = std::fs::remove_file(cache_path);
    }
}
