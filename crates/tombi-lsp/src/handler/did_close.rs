use tower_lsp::lsp_types::DidCloseTextDocumentParams;

use crate::Backend;

pub async fn handle_did_close(backend: &Backend, params: DidCloseTextDocumentParams) {
    tracing::info!("handle_did_close");
    tracing::trace!(?params);

    let DidCloseTextDocumentParams { text_document } = params;

    let text_document_uri = text_document.uri.into();
    let mut document_sources = backend.document_sources.write().await;

    document_sources.remove(&text_document_uri);
}
