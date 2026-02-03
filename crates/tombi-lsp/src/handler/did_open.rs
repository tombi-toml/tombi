use tower_lsp::lsp_types::DidOpenTextDocumentParams;

use crate::{backend::Backend, document::DocumentSource};

pub async fn handle_did_open(backend: &Backend, params: DidOpenTextDocumentParams) {
    log::info!("handle_did_open");
    log::trace!("{:?}", params);

    let DidOpenTextDocumentParams { text_document, .. } = params;

    let text_document_uri: tombi_uri::Uri = text_document.uri.into();
    let mut document_sources = backend.document_sources.write().await;
    let toml_version = backend
        .text_document_toml_version(&text_document_uri, &text_document.text)
        .await;

    document_sources.insert(
        text_document_uri.clone(),
        DocumentSource::new(
            text_document.text,
            Some(text_document.version),
            toml_version,
            backend.capabilities.read().await.encoding_kind,
        ),
    );
    drop(document_sources);

    // Publish diagnostics for the opened document
    backend.push_diagnostics(text_document_uri).await;
}
