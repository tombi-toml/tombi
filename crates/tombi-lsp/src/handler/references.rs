use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{ReferenceParams, TextDocumentPositionParams};

use crate::Backend;
use crate::config_manager::ConfigSchemaStore;
use crate::handler::hover::get_hover_keys_with_range;

pub async fn handle_references(
    backend: &Backend,
    params: ReferenceParams,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    log::info!("handle_references");
    log::trace!("{:?}", params);

    let ReferenceParams {
        text_document_position:
            TextDocumentPositionParams {
                text_document,
                position,
            },
        ..
    } = params;
    let text_document_uri = text_document.uri.into();

    let ConfigSchemaStore { config, .. } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    if !config
        .lsp
        .as_ref()
        .and_then(|server| server.references.as_ref())
        .and_then(|references| references.enabled)
        .unwrap_or_default()
        .value()
    {
        log::debug!("`server.references.enabled` is false");
        return Ok(None);
    }

    let Ok(document_sources) = backend.document_sources.try_read() else {
        return Ok(None);
    };
    let Some(document_source) = document_sources.get(&text_document_uri) else {
        return Ok(None);
    };

    let root = document_source.ast();
    let toml_version = document_source.toml_version;
    let line_index = document_source.line_index();

    let position = position.into_lsp(line_index);

    let Some((keys, _)) = get_hover_keys_with_range(&root, position, toml_version).await else {
        return Ok(None);
    };

    let document_tree = document_source.document_tree();
    let accessors = tombi_document_tree::get_accessors(&document_tree, &keys, position);

    if config.cargo_extension_enabled()
        && let Some(locations) = tombi_extension_cargo::references(
            &text_document_uri,
            &document_tree,
            &accessors,
            toml_version,
            config.cargo_extension_features(),
        )
        .await?
    {
        return Ok(Some(locations));
    }

    if config.pyproject_extension_enabled()
        && let Some(locations) = tombi_extension_pyproject::references(
            &text_document_uri,
            &document_tree,
            &accessors,
            toml_version,
            config.pyproject_extension_features(),
        )
        .await?
    {
        return Ok(Some(locations));
    }

    Ok(None)
}
