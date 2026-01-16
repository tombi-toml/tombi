use std::{collections::HashMap, path::PathBuf};

use itertools::Either;
use tombi_config::TomlVersion;
use tombi_glob::{MatchResult, matches_file_patterns};
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::{
    backend::Backend, config_manager::ConfigSchemaStore, document::DocumentSource,
    handler::TomlVersionSource,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_get_status(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<GetStatusResponse, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_get_status");
    tracing::trace!(?params);

    let TextDocumentIdentifier { uri } = params;
    let text_document_uri = uri.into();

    let ConfigSchemaStore {
        config,
        config_path,
        schema_store,
        ..
    } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    let (toml_version, source, schema, sub_schemas) = {
        let document_sources = backend.document_sources.read().await;
        if let Some(document_source) = document_sources.get(&text_document_uri) {
            let (toml_version, source) = backend
                .text_document_toml_version_and_source(&text_document_uri, document_source.text())
                .await;
            let (schema, sub_schemas) =
                get_schema_info(&text_document_uri, &document_source, &schema_store).await;

            (toml_version, source, schema, sub_schemas)
        } else {
            (
                TomlVersion::default(),
                TomlVersionSource::Default,
                None,
                None,
            )
        }
    };

    let mut ignore = None;
    if let Ok(text_document_path) = text_document_uri.to_file_path() {
        ignore = match matches_file_patterns(&text_document_path, config_path.as_deref(), &config) {
            MatchResult::IncludeNotMatched => Some(IgnoreReason::IncludeFilePatternNotMatched),
            MatchResult::ExcludeMatched => Some(IgnoreReason::ExcludeFilePatternMatched),
            MatchResult::Matched => None,
        };
    }

    Ok(GetStatusResponse {
        toml_version,
        source,
        config_path,
        ignore,
        schema,
        sub_schemas,
    })
}

async fn get_schema_info(
    text_document_uri: &tombi_uri::Uri,
    document_source: &DocumentSource,
    schema_store: &tombi_schema_store::SchemaStore,
) -> (Option<SchemaInfo>, Option<Vec<SubSchemaInfo>>) {
    let directive_schema_uri = document_source
        .ast()
        .schema_document_comment_directive(text_document_uri.to_file_path().ok().as_deref())
        .and_then(|directive| directive.uri.ok());

    let mut schema_by_uri: HashMap<tombi_uri::SchemaUri, tombi_schema_store::Schema> =
        HashMap::new();
    for schema in schema_store.list_schemas().await {
        match schema_by_uri.get_mut(&schema.schema_uri) {
            Some(existing) => {
                if schema.source == tombi_schema_store::SchemaSource::Config {
                    existing.title = schema.title;
                    existing.description = schema.description;
                } else {
                    *existing = schema;
                }
            }
            None => {
                schema_by_uri.insert(schema.schema_uri.clone(), schema);
            }
        }
    }

    let source_schema = schema_store
        .resolve_source_schema_from_ast(
            document_source.ast(),
            Some(Either::Left(text_document_uri)),
        )
        .await
        .ok()
        .flatten();

    let Some(source_schema) = source_schema else {
        return (None, None);
    };

    let mut schema = None;
    let mut sub_schemas = Vec::new();

    if let Some(root_schema) = source_schema.root_schema.as_ref() {
        let root_uri: tombi_uri::SchemaUri = root_schema.schema_uri.clone();
        let store_schema = schema_by_uri.get(&root_uri).cloned();

        let mut title = store_schema.as_ref().and_then(|s| s.title.clone());
        let description = store_schema.as_ref().and_then(|s| s.description.clone());

        if title.is_none() && Some(&root_uri) == directive_schema_uri.as_ref() {
            title = Some("Document Directive Schema".to_string());
        }

        schema = Some(SchemaInfo {
            title,
            description,
            path: root_uri.to_string(),
        });
    }

    for (root_key, sub_schema_uri) in source_schema.sub_schema_uri_map.iter() {
        let store_schema = schema_by_uri.get(sub_schema_uri).cloned();

        let title = store_schema.as_ref().and_then(|s| s.title.clone());
        let description = store_schema.as_ref().and_then(|s| s.description.clone());

        sub_schemas.push(SubSchemaInfo {
            title,
            description,
            root: tombi_schema_store::SchemaAccessors::from(root_key.clone()).to_string(),
            path: sub_schema_uri.to_string(),
        });
    }

    let sub_schemas_result = if sub_schemas.is_empty() {
        None
    } else {
        Some(sub_schemas)
    };

    (schema, sub_schemas_result)
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusResponse {
    pub toml_version: TomlVersion,
    pub source: TomlVersionSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<IgnoreReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_schemas: Option<Vec<SubSchemaInfo>>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubSchemaInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub root: String,
    pub path: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IgnoreReason {
    IncludeFilePatternNotMatched,
    ExcludeFilePatternMatched,
}
