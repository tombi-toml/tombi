use std::collections::HashMap;

use crate::{Backend, config_manager::ConfigSchemaStore, handler::list_schemas::SchemaInfo};

use itertools::Either;
use tower_lsp::lsp_types::TextDocumentIdentifier;

/// Normalize an optional text field by trimming whitespace and converting empty strings to None.
fn normalize_optional_text(input: Option<String>) -> Option<String> {
    input.and_then(|s| if s.trim().is_empty() { None } else { Some(s) })
}

fn schema_info_from_store(
    store_schema: Option<tombi_schema_store::Schema>,
    uri: tombi_uri::SchemaUri,
    toml_version: Option<String>,
) -> SchemaInfo {
    if let Some(store_schema) = store_schema {
        SchemaInfo {
            title: normalize_optional_text(store_schema.title),
            description: normalize_optional_text(store_schema.description),
            toml_version: store_schema
                .toml_version
                .map(|v| v.to_string())
                .or(toml_version),
            uri,
            catalog_uri: store_schema.catalog_uri,
        }
    } else {
        SchemaInfo {
            title: None,
            description: None,
            toml_version,
            uri,
            catalog_uri: None,
        }
    }
}

/// Handle the `tombi/getSchemas` request to list resolved schemas for a document.
///
/// This returns the *resolved* schema URIs for the given document (root schema plus
/// any referenced sub-schemas), based on the document contents and current schema-store.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_get_schemas(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<GetSchemasResponse, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_get_schemas");
    tracing::trace!(?params);

    let TextDocumentIdentifier { uri } = params;
    let text_document_uri: tombi_uri::Uri = uri.into();

    let ConfigSchemaStore { schema_store, .. } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    let document_sources = backend.document_sources.read().await;
    let Some(document_source) = document_sources.get(&text_document_uri) else {
        return Ok(GetSchemasResponse { schemas: vec![] });
    };

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
                    if normalize_optional_text(schema.title.clone()).is_some() {
                        existing.title = schema.title;
                    }
                    if normalize_optional_text(schema.description.clone()).is_some() {
                        existing.description = schema.description;
                    }
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
            Some(Either::Left(&text_document_uri)),
        )
        .await
        .ok()
        .flatten();

    let Some(source_schema) = source_schema else {
        return Ok(GetSchemasResponse { schemas: vec![] });
    };

    let mut schemas = Vec::new();

    if let Some(root_schema) = source_schema.root_schema.as_ref() {
        let root_uri: tombi_uri::SchemaUri = root_schema.schema_uri.clone();
        let mut schema_info = schema_info_from_store(
            schema_by_uri.get(&root_uri).cloned(),
            root_uri.clone(),
            root_schema.toml_version().map(|v| v.to_string()),
        );

        if schema_info.title.is_none() && Some(&root_uri) == directive_schema_uri.as_ref() {
            schema_info.title = Some("Document Directive Schema".to_string());
        }

        schemas.push(schema_info);
    }

    for sub_schema_uri in source_schema.sub_schema_uri_map.values() {
        if schemas
            .iter()
            .any(|s: &SchemaInfo| &s.uri == sub_schema_uri)
        {
            continue;
        }

        schemas.push(schema_info_from_store(
            schema_by_uri.get(sub_schema_uri).cloned(),
            sub_schema_uri.clone(),
            None,
        ));
    }

    Ok(GetSchemasResponse { schemas })
}

pub type GetSchemasParams = TextDocumentIdentifier;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemasResponse {
    pub schemas: Vec<SchemaInfo>,
}
