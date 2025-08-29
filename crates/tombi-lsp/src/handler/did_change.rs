use tower_lsp::lsp_types::DidChangeTextDocumentParams;

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_did_change(backend: &Backend, params: DidChangeTextDocumentParams) {
    tracing::info!("handle_did_change");
    tracing::trace!(?params);

    let DidChangeTextDocumentParams {
        text_document,
        content_changes,
    } = params;

    let text_document_uri = text_document.uri.into();
    let mut document_sources = backend.document_sources.write().await;
    let Some(document) = document_sources.get_mut(&text_document_uri) else {
        return;
    };

    for content_change in content_changes {
        if let Some(range) = content_change.range {
            tracing::warn!("Range change is not supported: {:?}", range);
        } else {
            document.text = content_change.text;
        }
    }
    drop(document_sources);

    // Publish diagnostics for the changed document
    backend
        .push_diagnostics(text_document_uri, Some(text_document.version))
        .await;
}
