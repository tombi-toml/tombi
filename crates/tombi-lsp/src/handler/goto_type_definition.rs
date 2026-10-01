use itertools::Either;
use tombi_schema_store::SchemaContext;
use tombi_text::IntoLsp;
use tower_lsp::lsp_types::request::GotoTypeDefinitionParams;

use crate::{
    backend::Backend,
    config_manager::ConfigSchemaStore,
    goto_type_definition::{
        SchemaLocation, TypeDefinition, get_tombi_document_comment_directive_type_definition,
        get_type_definition, location_key,
    },
    handler::hover::get_hover_keys_with_span,
};

fn type_definition_locations(
    type_definitions: Vec<TypeDefinition>,
    encoding: tombi_text::EncodingKind,
) -> Vec<SchemaLocation> {
    let mut unique_type_definitions: Vec<TypeDefinition> =
        Vec::with_capacity(type_definitions.len());
    for type_definition in type_definitions {
        if !unique_type_definitions.iter().any(|existing| {
            location_key(&existing.schema_base_uri, span_of(existing))
                == location_key(&type_definition.schema_base_uri, span_of(&type_definition))
        }) {
            unique_type_definitions.push(type_definition);
        }
    }
    unique_type_definitions
        .into_iter()
        .map(|type_definition| SchemaLocation {
            uri: type_definition.schema_base_uri.into(),
            range: type_definition
                .span
                .map(|span| span.range(encoding))
                .unwrap_or_default(),
        })
        .collect()
}

fn span_of(type_definition: &TypeDefinition) -> Option<tombi_text::Span> {
    type_definition.span.as_ref().map(|span| span.span)
}

pub async fn handle_goto_type_definition(
    backend: &Backend,
    params: GotoTypeDefinitionParams,
) -> Result<Option<Vec<SchemaLocation>>, tower_lsp::jsonrpc::Error> {
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

    log::info!("handle_goto_type_definition");

    let Some(document_source) = backend.document_source(&text_document_uri) else {
        return Ok(Default::default());
    };

    let root = document_source.ast();
    let toml_version = document_source.toml_version;
    let line_index = document_source.line_index();
    let encoding = document_source.encoding_kind();

    let offset: tombi_text::Offset = position.into_lsp(line_index, encoding);

    let type_definitions =
        get_tombi_document_comment_directive_type_definition(&root, offset).await;
    if !type_definitions.is_empty() {
        return Ok(Some(type_definition_locations(type_definitions, encoding)));
    }

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document_uri)))
        .await
        .ok()
        .flatten();

    let Some((keys, span)) =
        get_hover_keys_with_span(&root, document_source.decoded(), offset, toml_version).await
    else {
        return Ok(Default::default());
    };

    if keys.is_empty() && span.is_none() {
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

    let mut type_definitions = get_type_definition(
        document_source.document_tree(),
        crate::CursorPosition::new(offset, line_index),
        &keys,
        &schema_context,
    )
    .await;
    for type_definition in &mut type_definitions {
        if let Some(source_schema_uri) = schema_store
            .source_schema_uri(&type_definition.schema_base_uri)
            .await
        {
            type_definition.schema_base_uri = source_schema_uri;
        }
    }
    if type_definitions.is_empty() {
        Ok(None)
    } else {
        Ok(Some(type_definition_locations(type_definitions, encoding)))
    }
}
