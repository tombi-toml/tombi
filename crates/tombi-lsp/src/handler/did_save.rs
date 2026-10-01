use tower_lsp::lsp_types::DidSaveTextDocumentParams;

use crate::{
    backend::Backend,
    document::{DocumentSource, ParsedText},
};

pub async fn handle_did_save(backend: &Backend, params: DidSaveTextDocumentParams) {
    log::info!("handle_did_save");
    log::trace!("{:?}", params);

    let DidSaveTextDocumentParams {
        text_document,
        text,
    } = params;

    let text_document_uri = text_document.uri.into();

    let mut need_publish_diagnostics = { backend.is_diagnostic_mode_push().await };

    if let Some(text) = text {
        let mut document_sources = backend.document_sources.write().await;

        if need_publish_diagnostics
            && document_sources
                .get(&text_document_uri)
                .is_some_and(|document| document.text() == text)
        {
            need_publish_diagnostics = false;
        }

        let parsed = ParsedText::parse(text);
        let toml_version = backend
            .text_document_toml_version(&text_document_uri, &parsed.root())
            .await;

        if let Some(document) = document_sources.get_mut(&text_document_uri) {
            *document = DocumentSource::new(
                parsed,
                document.version,
                toml_version,
                document.encoding_kind(),
            );
        };
    };

    backend
        .workspace_diagnostics_cache
        .write()
        .await
        .clear(&text_document_uri);

    // Publish diagnostics for the saved document
    if need_publish_diagnostics {
        backend.push_diagnostics(text_document_uri).await
    }
}
