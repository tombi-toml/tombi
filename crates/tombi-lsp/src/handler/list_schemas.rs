use std::sync::Arc;

use crate::Backend;

/// Handle the `tombi/listSchemas` request to list all available schemas.
///
/// This function returns a list of all schemas known to the LSP server,
/// including their URIs, file match patterns, and other metadata.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_list_schemas(
    backend: &Backend,
    _params: ListSchemasParams,
) -> Result<ListSchemasResponse, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_list_schemas");

    let schemas = backend.config_manager.list_schemas().await;

    let schema_infos = schemas
        .into_iter()
        .map(|schema| SchemaInfo {
            title: schema.title,
            description: schema.description,
            toml_version: schema.toml_version.map(|v| v.to_string()),
            uri: schema.schema_uri,
            catalog_uri: schema.catalog_uri,
        })
        .collect();

    Ok(ListSchemasResponse {
        schemas: schema_infos,
    })
}

#[derive(Debug, serde::Deserialize)]
pub struct ListSchemasParams {}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSchemasResponse {
    pub schemas: Vec<SchemaInfo>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub toml_version: Option<String>,

    pub uri: tombi_uri::SchemaUri,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_uri: Option<Arc<tombi_uri::CatalogUri>>,
}
