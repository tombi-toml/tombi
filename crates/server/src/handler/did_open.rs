use tower_lsp::lsp_types::DidOpenTextDocumentParams;

use crate::{backend::Backend, document::DocumentSource};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_did_open(backend: &Backend, params: DidOpenTextDocumentParams) {
    tracing::info!("handle_did_open");
    tracing::trace!(?params);

    let DidOpenTextDocumentParams { text_document, .. } = params;

    let mut document_sources = backend.document_sources.write().await;
    document_sources.insert(
        text_document.uri,
        DocumentSource::new(text_document.text, text_document.version),
    );
}
