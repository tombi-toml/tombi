use tower_lsp::lsp_types::DidCloseTextDocumentParams;

use crate::{Backend, config_manager::ConfigSchemaStore};

pub async fn handle_did_close(backend: &Backend, params: DidCloseTextDocumentParams) {
    log::info!("handle_did_close");
    log::trace!("{:?}", params);

    let DidCloseTextDocumentParams { text_document } = params;

    let text_document_uri = text_document.uri.as_ref();

    {
        let mut document_sources = backend.document_sources.write().await;

        if let Some(document) = document_sources.get_mut(text_document_uri) {
            document.version = None;
        }
    }

    backend
        .workspace_diagnostics_cache
        .write()
        .await
        .close(text_document_uri);

    let ConfigSchemaStore { config, .. } = backend
        .config_manager
        .config_schema_store_for_uri(text_document_uri)
        .await;

    if !config
        .lsp
        .as_ref()
        .and_then(|server| server.workspace_diagnostic.as_ref())
        .and_then(|workspace_diagnostic| workspace_diagnostic.enabled)
        .unwrap_or_default()
        .value()
    {
        backend
            .client
            .publish_diagnostics(text_document.uri, Vec::new(), None)
            .await;
    }
}
