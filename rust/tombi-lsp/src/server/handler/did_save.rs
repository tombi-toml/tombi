use tower_lsp::lsp_types::DidSaveTextDocumentParams;

use crate::server::backend::Backend;

pub async fn handle_did_save(
    backend: &Backend,
    DidSaveTextDocumentParams {
        text_document,
        text,
    }: DidSaveTextDocumentParams,
) {
    tracing::info!("handle_did_save");

    if let Some(text) = text {
        if let Some(mut document) = backend.documents.get_mut(&text_document.uri) {
            document.source = text;
        }
    }
}