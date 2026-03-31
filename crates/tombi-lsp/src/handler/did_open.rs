use tower_lsp::lsp_types::DidOpenTextDocumentParams;

use crate::{backend::Backend, document::DocumentSource};

pub async fn handle_did_open(backend: &Backend, params: DidOpenTextDocumentParams) {
    log::info!("handle_did_open");
    log::trace!("{:?}", params);

    let DidOpenTextDocumentParams { text_document, .. } = params;

    let text_document_uri: tombi_uri::Uri = text_document.uri.into();
    let toml_version = backend
        .text_document_toml_version(&text_document_uri, &text_document.text)
        .await;
    let encoding_kind = backend.capabilities.read().await.encoding_kind;
    let document_source = DocumentSource::new(
        text_document.text,
        Some(text_document.version),
        toml_version,
        encoding_kind,
    );
    let document_tree = document_source.document_tree();

    let mut document_sources = backend.document_sources.write().await;

    document_sources.insert(text_document_uri.clone(), document_source);
    drop(document_sources);

    let config_schema_store = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;
    let offline = config_schema_store.schema_store.offline();
    let cache_options = config_schema_store.schema_store.cache_options();

    let mut cache_warming_handle: Option<tokio::task::JoinHandle<bool>> = None;

    if config_schema_store.config.cargo_extension_enabled()
        && let Ok(Some(handle)) = tombi_extension_cargo::did_open(
            &text_document_uri,
            document_tree.as_ref(),
            toml_version,
            offline,
            cache_options,
            config_schema_store.config.cargo_extension_features(),
        )
        .await
    {
        cache_warming_handle = Some(handle)
    } else if config_schema_store.config.pyproject_extension_enabled()
        && let Ok(Some(handle)) = tombi_extension_pyproject::did_open(
            &text_document_uri,
            document_tree.as_ref(),
            toml_version,
            offline,
            cache_options,
            config_schema_store.config.pyproject_extension_features(),
        )
        .await
    {
        cache_warming_handle = Some(handle)
    }

    // Publish diagnostics for the opened document
    backend.push_diagnostics(text_document_uri).await;

    if let Some(handle) = cache_warming_handle {
        let client = backend.client.clone();
        tokio::spawn(async move {
            let Ok(should_refresh_inlay_hint) = handle.await else {
                return;
            };

            if should_refresh_inlay_hint && let Err(err) = client.inlay_hint_refresh().await {
                log::debug!("Failed to request warmed inlay hint refresh: {err}");
            }
        });
    }
}
