use itertools::Either;
use tombi_config::TomlVersion;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::{backend::Backend, config_manager::ConfigSchemaStore};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_get_toml_version(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<GetTomlVersionResponse, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_get_toml_version");
    tracing::trace!(?params);

    let TextDocumentIdentifier { uri } = params;

    let ConfigSchemaStore {
        config,
        schema_store,
    } = backend
        .config_manager
        .config_schema_store_for_url(&uri)
        .await;

    let source_schema = match backend.get_incomplete_ast(&uri).await {
        Some(root) => schema_store
            .resolve_source_schema_from_ast(&root, Some(Either::Left(&uri)))
            .await
            .ok()
            .flatten(),
        None => None,
    };

    let (toml_version, source) = backend
        .source_toml_version(source_schema.as_ref(), &config)
        .await;

    Ok(GetTomlVersionResponse {
        toml_version,
        source,
    })
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTomlVersionResponse {
    pub toml_version: TomlVersion,
    pub source: TomlVersionSource,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TomlVersionSource {
    Config,
    Schema,
    Default,
}
