use tower_lsp::lsp_types::DidOpenTextDocumentParams;

use crate::{backend::Backend, document::DocumentSource};

fn select_cache_warming<T>(
    cargo_enabled: bool,
    cargo: impl FnOnce() -> Option<T>,
    pyproject_enabled: bool,
    pyproject: impl FnOnce() -> Option<T>,
) -> Option<T> {
    cargo_enabled
        .then(cargo)
        .flatten()
        .or_else(|| pyproject_enabled.then(pyproject).flatten())
}

pub async fn handle_did_open(backend: &Backend, params: DidOpenTextDocumentParams) {
    log::info!("handle_did_open");
    log::trace!("{:?}", params);

    let DidOpenTextDocumentParams { text_document, .. } = params;

    let text_document_uri: tombi_uri::Uri = text_document.uri.into();
    backend.begin_document_open(text_document_uri.clone());
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

    {
        let mut document_sources = backend.document_sources.write().await;

        document_sources.insert(text_document_uri.clone(), document_source);
    }

    backend
        .workspace_diagnostics_cache
        .write()
        .await
        .clear(&text_document_uri);

    let config_schema_store = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;
    let offline = config_schema_store.schema_store.offline();
    let cache_options = config_schema_store.schema_store.cache_options();

    let cache_warming = select_cache_warming(
        config_schema_store.config.cargo_extension_enabled(),
        || {
            tombi_extension_cargo::did_open(
                &text_document_uri,
                document_tree.as_ref(),
                toml_version,
                offline,
                cache_options,
                config_schema_store.config.cargo_extension_features(),
            )
        },
        config_schema_store.config.pyproject_extension_enabled(),
        || {
            tombi_extension_pyproject::did_open(
                &text_document_uri,
                document_tree.as_ref(),
                toml_version,
                offline,
                cache_options,
                config_schema_store.config.pyproject_extension_features(),
            )
        },
    );
    backend.finish_document_open(&text_document_uri);

    // Publish diagnostics for the opened document
    backend.push_diagnostics(text_document_uri).await;

    if let Some(cache_warming) = cache_warming {
        backend.spawn_background_task(cache_warming);
    }
}

#[cfg(test)]
mod tests {
    use super::select_cache_warming;

    #[test]
    fn cache_warming_falls_back_to_pyproject() {
        assert_eq!(
            select_cache_warming(true, || None, true, || Some("pyproject")),
            Some("pyproject")
        );
    }
}
