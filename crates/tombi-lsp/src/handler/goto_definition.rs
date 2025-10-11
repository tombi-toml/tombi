use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{GotoDefinitionParams, TextDocumentPositionParams};

use crate::config_manager::ConfigSchemaStore;
use crate::handler::hover::get_hover_keys_with_range;
use crate::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_goto_definition(
    backend: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<Vec<tombi_extension::DefinitionLocation>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_goto_definition");
    tracing::trace!(?params);

    let GotoDefinitionParams {
        text_document_position_params:
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
        .lsp()
        .and_then(|server| server.goto_definition.as_ref())
        .and_then(|goto_definition| goto_definition.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.goto_definition.enabled` is false");
        return Ok(Default::default());
    }

    let document_sources = backend.document_sources.read().await;
    let Some(document_source) = document_sources.get(&text_document_uri) else {
        return Ok(Default::default());
    };

    let root = document_source.ast();
    let toml_version = document_source.toml_version;
    let line_index = document_source.line_index();

    let position = position.into_lsp(line_index);

    if let Some(location) = resolve_schema_definition_location(root, &text_document_uri, position) {
        return Ok(Some(vec![location]));
    }

    let Some((keys, _)) = get_hover_keys_with_range(root, position, toml_version).await else {
        return Ok(Default::default());
    };

    let document_tree = document_source.document_tree();
    let accessors = tombi_document_tree::get_accessors(document_tree, &keys, position);

    if let Some(locations) = tombi_extension_cargo::goto_definition(
        &text_document_uri,
        document_tree,
        &accessors,
        toml_version,
    )
    .await?
    {
        return Ok(locations.into());
    }

    if let Some(locations) = tombi_extension_uv::goto_definition(
        &text_document_uri,
        document_tree,
        &accessors,
        toml_version,
    )
    .await?
    {
        return Ok(locations.into());
    }

    if let Some(locations) = tombi_extension_tombi::goto_definition(
        &text_document_uri,
        document_tree,
        &accessors,
        toml_version,
    )
    .await?
    {
        return Ok(locations.into());
    }

    Ok(Default::default())
}

fn resolve_schema_definition_location(
    root: &tombi_ast::Root,
    text_document_uri: &tombi_uri::Uri,
    position: tombi_text::Position,
) -> Option<tombi_extension::DefinitionLocation> {
    let document_file_path = text_document_uri.to_file_path().ok()?;
    let schema_directive =
        root.schema_document_comment_directive(Some(document_file_path.as_path()))?;

    if !schema_directive.uri_range.contains(position) {
        return None;
    }

    let Ok(uri) = schema_directive.uri else {
        return None;
    };

    if uri.scheme() == "file" {
        if let Ok(path) = uri.to_file_path() {
            if !path.is_file() {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(tombi_extension::DefinitionLocation {
        uri: uri.into(),
        range: tombi_text::Range::default(),
    })
}
