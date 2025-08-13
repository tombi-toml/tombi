use std::path::PathBuf;

use itertools::Either;
use tombi_config::TomlVersion;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::{backend::Backend, config_manager::ConfigSchemaStore, handler::TomlVersionSource};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_get_status(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<GetStatusResponse, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_get_status");
    tracing::trace!(?params);

    let TextDocumentIdentifier { uri } = params;

    let config_path = backend.config_manager.get_config_path_for_url(&uri).await;

    let ConfigSchemaStore {
        config,
        schema_store,
    } = backend
        .config_manager
        .config_schema_store_for_url(&uri)
        .await;

    let (root, source_schema) = match backend.get_incomplete_ast(&uri).await {
        Some(root) => {
            let source_schema = schema_store
                .resolve_source_schema_from_ast(&root, Some(Either::Left(&uri)))
                .await
                .ok()
                .flatten();
            (Some(root), source_schema)
        }
        None => (None, None),
    };

    let root_comment_directive = match root.as_ref() {
        Some(root) => tombi_comment_directive::get_tombi_comment_directive(root).await,
        None => None,
    };

    let (toml_version, source) = backend
        .source_toml_version(root_comment_directive, source_schema.as_ref(), &config)
        .await;

    Ok(GetStatusResponse {
        toml_version,
        source,
        config_path,
    })
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusResponse {
    pub toml_version: TomlVersion,
    pub source: TomlVersionSource,
    pub config_path: Option<PathBuf>,
}
