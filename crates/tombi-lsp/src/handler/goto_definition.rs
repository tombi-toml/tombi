use itertools::Either;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tower_lsp::lsp_types::{
    GotoDefinitionParams, GotoDefinitionResponse, TextDocumentPositionParams,
};

use crate::handler::hover::{get_hover_accessors, get_hover_keys_and_range};
use crate::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_goto_definition(
    backend: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>, tower_lsp::jsonrpc::Error> {
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

    let config = backend.config().await;

    if !config
        .lsp()
        .and_then(|server| server.goto_definition.as_ref())
        .and_then(|goto_definition| goto_definition.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.goto_definition.enabled` is false");
        return Ok(None);
    }

    let Some(root) = backend.get_incomplete_ast(&text_document.uri).await else {
        return Ok(None);
    };

    let source_schema = backend
        .schema_store
        .build_source_schema_from_ast(&root, Some(Either::Left(&text_document.uri)))
        .await
        .ok()
        .flatten();

    let (toml_version, _) = backend.source_toml_version(source_schema.as_ref()).await;

    let position = position.into();
    let Some((keys, _)) = get_hover_keys_and_range(&root, position, toml_version).await else {
        return Ok(None);
    };

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;
    let accessors = get_hover_accessors(&document_tree, &keys, position);

    if let Some(locations) = tombi_extension_cargo::goto_definition(
        &text_document,
        &document_tree,
        &accessors,
        toml_version,
    )
    .await?
    {
        return Ok(locations.into());
    }

    if let Some(locations) = tombi_extension_uv::goto_definition(
        &text_document,
        &document_tree,
        &accessors,
        toml_version,
    )
    .await?
    {
        return Ok(locations.into());
    }

    Ok(None)
}
