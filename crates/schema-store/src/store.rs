use std::{ops::Deref, sync::Arc};

use ahash::AHashMap;
use config::Schema;
use futures::{future::BoxFuture, FutureExt};
use itertools::{Either, Itertools};
use tokio::sync::RwLock;

use crate::{
    json::CatalogUrl, DocumentSchema, SchemaAccessor, SchemaAccessors, SchemaUrl, SourceSchema,
};

#[derive(Debug, Clone)]
pub struct SchemaStore {
    http_client: reqwest::Client,
    document_schemas:
        Arc<tokio::sync::RwLock<AHashMap<SchemaUrl, Result<DocumentSchema, crate::Error>>>>,
    schemas: Arc<RwLock<Vec<crate::Schema>>>,
    options: crate::Options,
}

impl SchemaStore {
    pub fn new(options: crate::Options) -> Self {
        Self {
            http_client: reqwest::Client::new(),
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

    pub async fn load_schemas(&self, schemas: &[Schema], base_dirpath: Option<&std::path::Path>) {
        let mut store_schemas = self.schemas.write().await;

        for schema in schemas.iter() {
            let schema_url = if let Ok(schema_url) = SchemaUrl::parse(schema.path()) {
                schema_url
            } else if let Ok(schema_url) = match base_dirpath {
                Some(base_dirpath) => SchemaUrl::from_file_path(base_dirpath.join(schema.path())),
                None => SchemaUrl::from_file_path(schema.path()),
            } {
                schema_url
            } else {
                tracing::error!("invalid schema path: {}", schema.path());
                continue;
            };

            tracing::debug!("load config schema from: {}", schema_url);

            store_schemas.push(crate::Schema {
                url: schema_url,
                include: schema.include().to_vec(),
                toml_version: schema.toml_version(),
                sub_root_keys: schema.root_keys().and_then(SchemaAccessor::parse),
            });
        }
    }

    pub async fn load_schemas_from_catalog_url(
        &self,
        catalog_url: &CatalogUrl,
    ) -> Result<(), crate::Error> {
        if matches!(catalog_url.scheme(), "http" | "https") && self.offline() {
            tracing::debug!("offline mode, skip fetch catalog from url: {}", catalog_url);
            return Ok(());
        }

        tracing::debug!("loading schema catalog: {}", catalog_url);

        if let Ok(response) = self.http_client.get(catalog_url.as_str()).send().await {
            match response.json::<crate::json::JsonCatalog>().await {
                Ok(catalog) => {
                    let mut schemas = self.schemas.write().await;
                    for schema in catalog.schemas {
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
                Err(err) => Err(crate::Error::InvalidJsonFormat {
                    url: catalog_url.deref().clone(),
                    reason: err.to_string(),
                }),
            }
        } else {
            Err(crate::Error::CatalogUrlFetchFailed {
                catalog_url: catalog_url.clone(),
            })
        }
    }

    pub async fn update_schema(&self, schema_url: &SchemaUrl) -> Result<bool, crate::Error> {
        if matches!(schema_url.scheme(), "http" | "https") && self.offline() {
            tracing::debug!("offline mode, skip fetch schema from url: {}", schema_url);
            return Ok(false);
        }

        let has_key = { self.document_schemas.read().await.contains_key(schema_url) };
        if has_key {
            self.document_schemas.write().await.insert(
                schema_url.clone(),
                self.try_fetch_document_schema_from_url(schema_url).await,
            );

            tracing::debug!("update schema: {}", schema_url);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn try_fetch_document_schema_from_url(
        &self,
        schema_url: &SchemaUrl,
    ) -> Result<DocumentSchema, crate::Error> {
        let schema: serde_json::Map<String, serde_json::Value> = match schema_url.scheme() {
            "file" => {
                let schema_path =
                    schema_url
                        .to_file_path()
                        .map_err(|_| crate::Error::InvalidSchemaUrl {
                            schema_url: schema_url.to_string(),
                        })?;
                let file = std::fs::File::open(&schema_path)
                    .map_err(|_| crate::Error::SchemaFileReadFailed { schema_path })?;

                serde_json::from_reader(file)
            }
            "http" | "https" => {
                assert!(
                    !self.offline(),
                    "offline mode, store don't have online schema url: {schema_url}",
                );

                tracing::debug!("fetch schema from url: {}", schema_url);

                let response = self
                    .http_client
                    .get(schema_url.as_ref())
                    .send()
                    .await
                    .map_err(|err| crate::Error::SchemaFetchFailed {
                        schema_url: schema_url.clone(),
                        reason: err.to_string(),
                    })?;

                let bytes =
                    response
                        .bytes()
                        .await
                        .map_err(|err| crate::Error::SchemaFetchFailed {
                            schema_url: schema_url.clone(),
                            reason: err.to_string(),
                        })?;

                serde_json::from_reader(std::io::Cursor::new(bytes))
            }
            _ => {
                return Err(crate::Error::UnsupportedSchemaUrl {
                    schema_url: schema_url.to_owned(),
                })
            }
        }
        .map_err(|err| crate::Error::SchemaFileParseFailed {
            schema_url: schema_url.to_owned(),
            reason: err.to_string(),
        })?;

        Ok(DocumentSchema::new(schema, schema_url.clone()))
    }

    pub fn try_get_document_schema<'a: 'b, 'b>(
        &'a self,
        schema_url: &'a SchemaUrl,
    ) -> BoxFuture<'b, Result<Option<DocumentSchema>, crate::Error>> {
        async move {
            if self.offline() && matches!(schema_url.scheme(), "http" | "https") {
                return Ok(None);
            }

            if let Some(document_schema) = self.document_schemas.read().await.get(schema_url) {
                return match document_schema {
                    Ok(document_schema) => Ok(Some(document_schema.clone())),
                    Err(err) => Err(err.clone()),
                };
            }

            self.document_schemas.write().await.insert(
                schema_url.clone(),
                self.try_fetch_document_schema_from_url(schema_url).await,
            );

            self.try_get_document_schema(schema_url).await
        }
        .boxed()
    }

    pub async fn try_get_source_schema_from_ast(
        &self,
        root: &ast::Root,
        source_url_or_path: Option<Either<&url::Url, &std::path::Path>>,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        let source_path = match source_url_or_path {
            Some(Either::Left(url)) => match url.scheme() {
                "file" => url.to_file_path().ok(),
                _ => None,
            },
            Some(Either::Right(path)) => Some(path.to_path_buf()),
            None => None,
        };

        if let Some(comments) = itertools::chain!(
            root.begin_dangling_comments()
                .into_iter()
                .next()
                .map(|comment| {
                    comment
                        .into_iter()
                        .map(|comment| ast::Comment::from(comment))
                        .collect_vec()
                }),
            root.dangling_comments().into_iter().next().map(|comment| {
                comment
                    .into_iter()
                    .map(|comment| ast::Comment::from(comment))
                    .collect_vec()
            })
        )
        .find(|comments| !comments.is_empty())
        {
            for comment in comments {
                if let Some(schema_url) = comment.schema_url(source_path.as_deref()) {
                    if let Ok(source_schema) = self
                        .try_get_source_schema_from_schema_url(&SchemaUrl::new(schema_url))
                        .await
                    {
                        return Ok(source_schema);
                    }
                }
            }
        }

        Ok(None)
    }

    #[inline]
    pub async fn try_get_source_schema_from_schema_url(
        &self,
        schema_url: &SchemaUrl,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        if self.offline() && matches!(schema_url.scheme(), "http" | "https") {
            tracing::debug!("offline mode, skip fetch schema: {}", schema_url);
            return Ok(None);
        }

        Ok(Some(SourceSchema {
            root_schema: self.try_get_document_schema(&schema_url).await?,
            sub_schema_url_map: Default::default(),
        }))
    }

    pub async fn try_get_source_schema_from_url(
        &self,
        source_url: &url::Url,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        match source_url.scheme() {
            "file" => {
                let source_path =
                    source_url
                        .to_file_path()
                        .map_err(|_| crate::Error::SourceUrlParseFailed {
                            source_url: source_url.to_owned(),
                        })?;
                self.try_get_source_schema_from_path(&source_path).await
            }
            "http" | "https" => {
                if self.offline() {
                    tracing::debug!("offline mode, skip fetch source from url: {}", source_url);
                    return Ok(None);
                }
                self.try_get_source_schema_from_schema_url(&SchemaUrl::new(source_url.clone()))
                    .await
            }
            "untitled" => Ok(None),
            _ => Err(crate::Error::SourceUrlUnsupported {
                source_url: source_url.to_owned(),
            }),
        }
    }

    pub async fn try_get_source_schema_from_path(
        &self,
        source_path: &std::path::Path,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        let schemas = self.schemas.read().await;
        let matching_schemas = schemas
            .iter()
            .filter(|catalog| {
                catalog.include.iter().any(|pat| {
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

    pub async fn try_get_source_schema(
        &self,
        source_url_or_path: Either<&url::Url, &std::path::Path>,
    ) -> Result<Option<SourceSchema>, crate::Error> {
        match source_url_or_path {
            Either::Left(source_url) => self.try_get_source_schema_from_url(source_url).await,
            Either::Right(source_path) => self.try_get_source_schema_from_path(source_path).await,
        }
        .inspect(|source_schema| {
            if let Some(source_schema) = source_schema {
                if let Some(root_schema) = &source_schema.root_schema {
                    tracing::debug!("find root schema from {}", root_schema.schema_url);
                }
                for (accessors, schema_url) in &source_schema.sub_schema_url_map {
                    tracing::debug!(
                        "find sub schema {:?} from {}",
                        SchemaAccessors::new(accessors.clone()),
                        schema_url
                    );
                }
            }
        })
    }
}
