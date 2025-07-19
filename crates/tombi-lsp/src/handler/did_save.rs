use tower_lsp::lsp_types::DidSaveTextDocumentParams;

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_did_save(backend: &Backend, params: DidSaveTextDocumentParams) {
    tracing::info!("handle_did_save");
    tracing::trace!(?params);

    let DidSaveTextDocumentParams {
        text_document,
        text,
    } = params;

    if let Some(text) = text {
        let mut document_sources = backend.document_sources.write().await;
        if let Some(document) = document_sources.get_mut(&text_document.uri) {
            document.text = text;
        }
        drop(document_sources);
    }

    // Publish diagnostics for the saved document
    backend.publish_diagnostics(text_document.uri, None).await
}
