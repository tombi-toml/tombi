use itertools::Either;
use tombi_schema_store::SchemaContext;
use tombi_text::IntoLsp;
use tower_lsp::lsp_types::request::GotoTypeDefinitionParams;

use crate::{
    backend::Backend,
    config_manager::ConfigSchemaStore,
    goto_type_definition::{
        TypeDefinition, get_tombi_document_comment_directive_type_definition, get_type_definition,
        location_key,
    },
    handler::hover::get_hover_keys_with_range,
};

fn type_definition_locations(
    type_definitions: Vec<TypeDefinition>,
) -> Vec<tombi_extension::Location> {
    let mut unique_type_definitions: Vec<TypeDefinition> =
        Vec::with_capacity(type_definitions.len());
    for type_definition in type_definitions {
        if !unique_type_definitions.iter().any(|existing| {
            location_key(&existing.schema_uri, existing.range)
                == location_key(&type_definition.schema_uri, type_definition.range)
        }) {
            unique_type_definitions.push(type_definition);
        }
    }
    unique_type_definitions
        .into_iter()
        .map(|type_definition| tombi_extension::Location {
            uri: type_definition.schema_uri.into(),
            range: type_definition.range,
        })
        .collect()
}

pub async fn handle_goto_type_definition(
    backend: &Backend,
    params: GotoTypeDefinitionParams,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    log::info!("handle_goto_type_definition");
    log::trace!("{:?}", params);

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
        .lsp
        .as_ref()
        .and_then(|server| server.goto_type_definition.as_ref())
        .and_then(|goto_type_definition| goto_type_definition.enabled)
        .unwrap_or_default()
        .value()
    {
        log::debug!("`server.goto_type_definition.enabled` is false");
        return Ok(Default::default());
    }

    let Ok(document_sources) = backend.document_sources.try_read() else {
        return Ok(Default::default());
    };
    let Some(document_source) = document_sources.get(&text_document_uri) else {
        return Ok(Default::default());
    };

    let root = document_source.ast();
    let toml_version = document_source.toml_version;
    let line_index = document_source.line_index();

    let position = position.into_lsp(line_index);

    let type_definitions =
        get_tombi_document_comment_directive_type_definition(&root, position).await;
    if !type_definitions.is_empty() {
        return Ok(Some(type_definition_locations(type_definitions)));
    }

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document_uri)))
        .await
        .ok()
        .flatten();

    let Some((keys, range)) = get_hover_keys_with_range(&root, position, toml_version).await else {
        return Ok(Default::default());
    };

    if keys.is_empty() && range.is_none() {
        return Ok(Default::default());
    }

    let strict = tombi_validator::comment_directive::get_tombi_document_comment_directive(&root)
        .await
        .and_then(|directive| directive.schema.and_then(|schema| schema.strict));
    let schema_context = SchemaContext::from_source_schema(
        toml_version,
        source_schema.as_ref(),
        &schema_store,
        strict,
    );

    let type_definitions = get_type_definition(
        &document_source.document_tree(),
        position,
        &keys,
        &schema_context,
    )
    .await;
    if type_definitions.is_empty() {
        Ok(None)
    } else {
        Ok(Some(type_definition_locations(type_definitions)))
    }
}
