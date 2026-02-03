use tower_lsp::lsp_types::DidCloseTextDocumentParams;

use crate::Backend;

pub async fn handle_did_close(backend: &Backend, params: DidCloseTextDocumentParams) {
    log::info!("handle_did_close");
    log::trace!("{:?}", params);

    let DidCloseTextDocumentParams { text_document } = params;

    let text_document_uri = text_document.uri.into();
    let mut document_sources = backend.document_sources.write().await;

    if let Some(document) = document_sources.get_mut(&text_document_uri) {
        document.version = None;
    }
}
