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

    let text_document_uri = text_document.uri.into();

    let mut need_publish_diagnostics = { backend.is_diagnostic_mode_push().await };

    if let Some(text) = text {
        let mut document_sources = backend.document_sources.write().await;

        let toml_version = backend
            .text_document_toml_version(&text_document_uri, &text)
            .await;

        if let Some(document) = document_sources.get_mut(&text_document_uri) {
            if need_publish_diagnostics && document.text() == &text {
                need_publish_diagnostics = false;
            }

            document.set_text(text, toml_version);
        };
        drop(document_sources);
    };

    // Publish diagnostics for the saved document
    if need_publish_diagnostics {
        backend.push_diagnostics(text_document_uri).await
    }
}
