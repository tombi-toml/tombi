use tower_lsp::lsp_types::DidChangeTextDocumentParams;

use crate::backend::Backend;

pub async fn handle_did_change(backend: &Backend, params: DidChangeTextDocumentParams) {
    tracing::info!("handle_did_change");
    tracing::trace!("{:?}", params);

    let DidChangeTextDocumentParams {
        text_document,
        content_changes,
    } = params;

    let text_document_uri = text_document.uri.into();
    let mut latest_text = None;

    for content_change in content_changes {
        if let Some(range) = content_change.range {
            tracing::warn!("Range change is not supported: {:?}", range);
        } else {
            latest_text = Some(content_change.text);
        }
    }

    // Apply the edit and bump the document version up front, without awaiting in
    // between, so that concurrently-processed requests (most importantly pull
    // diagnostics) observe the new content immediately instead of a stale
    // snapshot. Resolving the TOML version touches the schema store (async), which
    // would otherwise yield before the document is updated and let a pull compute
    // diagnostics against the previous version. The TOML version is reused from the
    // previous parse here and refined below only if the edit actually changed it
    // (e.g. an edited `#:schema` directive).
    let (need_publish_diagnostics, previous_toml_version) = {
        let mut document_sources = backend.document_sources.write().await;
        let Some(document) = document_sources.get_mut(&text_document_uri) else {
            return;
        };

        let need_publish_diagnostics = document
            .version
            .is_none_or(|version| version < text_document.version);
        let previous_toml_version = document.toml_version;

        if let Some(text) = latest_text.as_ref() {
            document.set_text(text, previous_toml_version);
        }
        document.version = Some(text_document.version);

        (need_publish_diagnostics, previous_toml_version)
    };

    backend
        .workspace_diagnostics_cache
        .write()
        .await
        .clear(&text_document_uri);

    // Refine the TOML version if this edit changed it, and re-apply only when the
    // document has not been superseded by a newer change in the meantime.
    if let Some(text) = latest_text.as_ref() {
        let toml_version = backend
            .text_document_toml_version(&text_document_uri, text)
            .await;

        if toml_version != previous_toml_version {
            {
                let mut document_sources = backend.document_sources.write().await;
                let Some(document) = document_sources.get_mut(&text_document_uri) else {
                    return;
                };
                if document.version != Some(text_document.version) {
                    return;
                }
                document.set_text(text, toml_version);
            }

            backend
                .workspace_diagnostics_cache
                .write()
                .await
                .clear(&text_document_uri);
        }
    }

    if need_publish_diagnostics {
        // Publish diagnostics for the changed document
        backend.push_diagnostics(text_document_uri).await;
    }
}
