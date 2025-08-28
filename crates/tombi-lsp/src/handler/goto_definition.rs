use itertools::Either;
use tombi_document_tree::IntoDocumentTreeAndErrors;
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

    let ConfigSchemaStore {
        config,
        schema_store,
        ..
    } = backend
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

    let Some(root) = backend.get_incomplete_ast(&text_document_uri).await else {
        return Ok(Default::default());
    };

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document_uri)))
        .await
        .ok()
        .flatten();

    let tombi_document_comment_directive =
        tombi_validator::comment_directive::get_tombi_document_comment_directive(&root).await;
    let (toml_version, _) = backend
        .source_toml_version(
            tombi_document_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let position = position.into();
    let Some((keys, _)) = get_hover_keys_with_range(&root, position, toml_version).await else {
        return Ok(Default::default());
    };

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;
    let accessors = tombi_document_tree::get_accessors(&document_tree, &keys, position);

    if let Some(locations) = tombi_extension_cargo::goto_definition(
        &text_document_uri,
        &document_tree,
        &accessors,
        toml_version,
    )
    .await?
    {
        return Ok(locations.into());
    }

    if let Some(locations) = tombi_extension_uv::goto_definition(
        &text_document_uri,
        &document_tree,
        &accessors,
        toml_version,
    )
    .await?
    {
        return Ok(locations.into());
    }

    if let Some(locations) = tombi_extension_tombi::goto_definition(
        &text_document_uri,
        &document_tree,
        &accessors,
        toml_version,
    )
    .await?
    {
        return Ok(locations.into());
    }

    Ok(Default::default())
}
