use tower_lsp::lsp_types::DidOpenTextDocumentParams;

use crate::{backend::Backend, document::DocumentSource};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_did_open(backend: &Backend, params: DidOpenTextDocumentParams) {
    tracing::info!("handle_did_open");
    tracing::trace!(?params);

    let DidOpenTextDocumentParams { text_document, .. } = params;

    let text_document_uri: tombi_uri::Uri = text_document.uri.into();
    let mut document_sources = backend.document_sources.write().await;
    document_sources.insert(
        text_document_uri.clone(),
        DocumentSource::new(text_document.text, Some(text_document.version)),
    );
    drop(document_sources);

    // Publish diagnostics for the opened document
    backend
        .push_diagnostics(text_document_uri, Some(text_document.version))
        .await;
}
