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
    let mut latest_content_change = None;

    for content_change in content_changes {
        if let Some(range) = content_change.range {
            log::warn!("Range change is not supported: {:?}", range);
        } else {
            let toml_version = backend
                .text_document_toml_version(&text_document_uri, &content_change.text)
                .await;

            latest_content_change = Some((content_change.text, toml_version));
        }
    }

    let need_publish_diagnostics = {
        let mut document_sources = backend.document_sources.write().await;
        let Some(document) = document_sources.get_mut(&text_document_uri) else {
            return;
        };

        let need_publish_diagnostics = document
            .version
            .is_none_or(|version| version < text_document.version);

        if let Some((text, toml_version)) = latest_content_change {
            document.set_text(text, toml_version);
        }
        document.version = Some(text_document.version);

        need_publish_diagnostics
    };

    backend
        .workspace_diagnostics_cache
        .write()
        .await
        .clear(&text_document_uri);

    if need_publish_diagnostics {
        // Publish diagnostics for the changed document
        backend.push_diagnostics(text_document_uri).await;
    }
}
