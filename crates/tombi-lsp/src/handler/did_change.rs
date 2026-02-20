use tower_lsp::lsp_types::DidChangeTextDocumentParams;

use crate::backend::Backend;

pub async fn handle_did_change(backend: &Backend, params: DidChangeTextDocumentParams) {
    log::info!("handle_did_change");
    log::trace!("{:?}", params);

    let DidChangeTextDocumentParams {
        text_document,
        content_changes,
    } = params;

    let text_document_uri = text_document.uri.into();
    let mut full_text_changes = Vec::new();
    for content_change in content_changes {
        if let Some(range) = content_change.range {
            log::warn!("Range change is not supported: {:?}", range);
        } else {
            full_text_changes.push(content_change.text);
        }
    }

    let mut changes_with_versions = Vec::with_capacity(full_text_changes.len());
    for text in full_text_changes {
        let toml_version = backend
            .text_document_toml_version(&text_document_uri, &text)
            .await;
        changes_with_versions.push((text, toml_version));
    }

    let mut document_sources = backend.document_sources.write().await;
    let Some(document) = document_sources.get_mut(&text_document_uri) else {
        return;
    };

    let need_publish_diagnostics = document
        .version
        .is_none_or(|version| version < text_document.version);

    for (text, toml_version) in changes_with_versions {
        document.set_text(text, toml_version);
    }

    document.version = Some(text_document.version);

    drop(document_sources);

    if need_publish_diagnostics {
        // Publish diagnostics for the changed document
        backend.push_diagnostics(text_document_uri).await;
    }
}
