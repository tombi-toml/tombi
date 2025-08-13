use itertools::Either;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_schema_store::get_accessors;
use tower_lsp::lsp_types::request::GotoDeclarationParams;
use tower_lsp::lsp_types::TextDocumentPositionParams;

use crate::config_manager::ConfigSchemaStore;
use crate::handler::hover::get_hover_keys_with_range;
use crate::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_goto_declaration(
    backend: &Backend,
    params: GotoDeclarationParams,
) -> Result<Option<Vec<tombi_extension::DefinitionLocation>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_goto_declaration");
    tracing::trace!(?params);

    let GotoDeclarationParams {
        text_document_position_params:
            TextDocumentPositionParams {
                text_document,
                position,
            },
        ..
    } = params;

    let ConfigSchemaStore {
        config,
        schema_store,
    } = backend
        .config_manager
        .config_schema_store_for_url(&text_document.uri)
        .await;

    if !config
        .lsp()
        .and_then(|server| server.goto_declaration.as_ref())
        .and_then(|goto_declaration| goto_declaration.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.goto_declaration.enabled` is false");
        return Ok(None);
    }

    let Some(root) = backend.get_incomplete_ast(&text_document.uri).await else {
        return Ok(None);
    };

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document.uri)))
        .await
        .ok()
        .flatten();

    let position = position.into();

    let document_tombi_comment_directive =
        tombi_comment_directive::get_document_tombi_comment_directive(&root).await;
    let (toml_version, _) = backend
        .source_toml_version(
            document_tombi_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let Some((keys, _)) = get_hover_keys_with_range(&root, position, toml_version).await else {
        return Ok(None);
    };

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;
    let accessors = get_accessors(&document_tree, &keys, position);

    if let Some(locations) = tombi_extension_cargo::goto_declaration(
        &text_document,
        &document_tree,
        &accessors,
        toml_version,
    )
    .await?
    {
        return Ok(locations.into());
    }

    if let Some(locations) = tombi_extension_uv::goto_declaration(
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
