use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{GotoDefinitionParams, TextDocumentPositionParams};

use crate::Backend;
use crate::config_manager::ConfigSchemaStore;
use crate::handler::hover::get_hover_keys_with_span;

pub async fn handle_goto_definition(
    backend: &Backend,
    params: GotoDefinitionParams,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    log::trace!("{:?}", params);

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
        .lsp
        .as_ref()
        .and_then(|server| server.goto_definition.as_ref())
        .and_then(|goto_definition| goto_definition.enabled)
        .unwrap_or_default()
        .value()
    {
        log::debug!("`server.goto_definition.enabled` is false");
        return Ok(Default::default());
    }

    log::info!("handle_goto_definition");

    let Some(document_source) = backend.document_source(&text_document_uri) else {
        return Ok(Default::default());
    };
    let converter = tombi_extension::SpanConverter::new(
        document_source.line_index(),
        document_source.encoding_kind(),
    );

    let root = document_source.ast();
    let toml_version = document_source.toml_version;
    let line_index = document_source.line_index();
    let encoding = document_source.encoding_kind();

    let offset: tombi_text::Offset = position.into_lsp(line_index, encoding);

    if let Some(location) = resolve_schema_location(&root, &text_document_uri, offset) {
        return Ok(Some(vec![location]));
    }

    let Some((keys, _)) =
        get_hover_keys_with_span(&root, document_source.decoded(), offset, toml_version).await
    else {
        return Ok(Default::default());
    };

    let document_tree = document_source.document_tree();
    let accessors = tombi_document_tree_syntax::get_accessors(document_tree, &keys, offset);

    if config.cargo_extension_enabled()
        && let Some(locations) = tombi_extension_cargo::goto_definition(
            &text_document_uri,
            document_tree,
            &accessors,
            toml_version,
            converter,
            config.cargo_extension_features(),
        )
        .await?
    {
        return Ok(locations.into());
    }

    if config.nagi_sql_extension_enabled()
        && let Some(locations) = tombi_extension_nagi_sql::goto_definition(
            &text_document_uri,
            document_tree,
            &accessors,
            toml_version,
            converter,
            config.nagi_sql_extension_features(),
        )
        .await?
    {
        return Ok(locations.into());
    }

    if config.pyproject_extension_enabled()
        && let Some(locations) = tombi_extension_pyproject::goto_definition(
            &text_document_uri,
            document_tree,
            &accessors,
            toml_version,
            converter,
            config.pyproject_extension_features(),
        )
        .await?
    {
        return Ok(locations.into());
    }

    if config.tombi_extension_enabled()
        && let Some(locations) = tombi_extension_tombi::goto_definition(
            &text_document_uri,
            document_tree,
            &accessors,
            toml_version,
            config.tombi_extension_features(),
        )
        .await?
    {
        return Ok(locations.into());
    }

    Ok(Default::default())
}

fn resolve_schema_location(
    root: &tombi_ast_syntax::Root<'_>,
    text_document_uri: &tombi_uri::Uri,
    offset: tombi_text::Offset,
) -> Option<tombi_extension::Location> {
    let document_file_path = text_document_uri.to_file_path().ok()?;
    let schema_directive =
        root.schema_document_comment_directive(Some(document_file_path.as_path()))?;

    if !schema_directive.uri_span.contains_inclusive(offset) {
        return None;
    }

    let Ok(uri) = schema_directive.uri else {
        return None;
    };

    if uri.scheme() == "file" {
        if let Ok(path) = uri.to_file_path() {
            if !tombi_fs::is_file(&path) {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(tombi_extension::Location {
        uri: uri.into(),
        range: None,
    })
}
