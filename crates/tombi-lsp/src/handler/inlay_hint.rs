use tombi_text::FromLsp;
use tower_lsp::lsp_types::InlayHintParams;

use crate::{Backend, config_manager::ConfigSchemaStore, document::DocumentSource};

pub async fn handle_inlay_hint(
    backend: &Backend,
    params: InlayHintParams,
) -> Result<Option<(Vec<tombi_extension::InlayHint>, DocumentSource)>, tower_lsp::jsonrpc::Error> {
    log::info!("handle_inlay_hint");
    log::trace!("{:?}", params);

    let InlayHintParams {
        text_document,
        range,
        ..
    } = params;
    let text_document_uri = text_document.uri.into();
    backend.wait_for_document_open(&text_document_uri).await;

    let ConfigSchemaStore {
        config,
        schema_store,
        ..
    } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    let Some(document_source) = backend.document_source(&text_document_uri) else {
        return Ok(None);
    };
    let document_tree = document_source.document_tree();
    let toml_version = document_source.toml_version;
    let visible_span = tombi_text::Span::from_lsp(
        range,
        document_source.line_index(),
        document_source.encoding_kind(),
    );

    if config.cargo_inlay_hint_enabled()
        && let Some(hints) = tombi_extension_cargo::inlay_hint(
            &text_document_uri,
            document_tree,
            visible_span,
            toml_version,
            schema_store.offline(),
            schema_store.cache_options(),
            config.cargo_extension_features(),
        )
        .await?
    {
        return Ok(Some((hints, document_source)));
    }

    if config.pyproject_inlay_hint_enabled()
        && let Some(hints) = tombi_extension_pyproject::inlay_hint(
            &text_document_uri,
            document_tree,
            visible_span,
            toml_version,
            config.pyproject_extension_features(),
        )
        .await?
    {
        return Ok(Some((hints, document_source)));
    }

    Ok(None)
}
