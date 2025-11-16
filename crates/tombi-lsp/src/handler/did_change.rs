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

    let need_publish_diagnostics = document
        .version
        .map_or(true, |version| version < text_document.version);

    for content_change in content_changes {
        if let Some(range) = content_change.range {
            tracing::warn!("Range change is not supported: {:?}", range);
        } else {
            let toml_version = backend
                .text_document_toml_version(&text_document_uri, &content_change.text)
                .await;

            document.set_text(content_change.text, toml_version);
        }
    }
    document.version = Some(text_document.version);

    drop(document_sources);

    if need_publish_diagnostics {
        // Publish diagnostics for the changed document
        backend.push_diagnostics(text_document_uri).await;
    }
}
