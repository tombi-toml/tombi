use itertools::Either;
use tombi_schema_store::SchemaContext;
use tombi_text::IntoLsp;
use tower_lsp::lsp_types::request::GotoTypeDefinitionParams;

use crate::{
    backend::Backend,
    config_manager::ConfigSchemaStore,
    goto_type_definition::{
        get_tombi_document_comment_directive_type_definition, get_type_definition, TypeDefinition,
    },
    handler::hover::get_hover_keys_with_range,
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_goto_type_definition(
    backend: &Backend,
    params: GotoTypeDefinitionParams,
) -> Result<Option<Vec<tombi_extension::DefinitionLocation>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_goto_type_definition");
    tracing::trace!(?params);

    let GotoTypeDefinitionParams {
        text_document_position_params:
            tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document,
                position,
                ..
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
        .and_then(|server| server.goto_type_definition.as_ref())
        .and_then(|goto_type_definition| goto_type_definition.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.goto_type_definition.enabled` is false");
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

    if let Some(type_definition) =
        get_tombi_document_comment_directive_type_definition(root, position).await
    {
        return Ok(Some(vec![tombi_extension::DefinitionLocation {
            uri: type_definition.schema_uri.into(),
            range: type_definition.range,
        }]));
    }

    let source_schema = schema_store
        .resolve_source_schema_from_ast(root, Some(Either::Left(&text_document_uri)))
        .await
        .ok()
        .flatten();

    let Some((keys, range)) = get_hover_keys_with_range(root, position, toml_version).await else {
        return Ok(Default::default());
    };

    if keys.is_empty() && range.is_none() {
        return Ok(Default::default());
    }

    Ok(
        match get_type_definition(
            document_source.document_tree(),
            position,
            &keys,
            &SchemaContext {
                toml_version,
                root_schema: source_schema.as_ref().and_then(|s| s.root_schema.as_ref()),
                sub_schema_uri_map: source_schema.as_ref().map(|s| &s.sub_schema_uri_map),
                store: &schema_store,
                strict: None,
            },
        )
        .await
        {
            Some(TypeDefinition {
                schema_uri, range, ..
            }) => Some(vec![tombi_extension::DefinitionLocation {
                uri: schema_uri.into(),
                range,
            }]),
            _ => Default::default(),
        },
    )
}
