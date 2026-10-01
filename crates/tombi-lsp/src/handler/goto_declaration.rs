use tombi_text::IntoLsp;
use tower_lsp::lsp_types::TextDocumentPositionParams;
use tower_lsp::lsp_types::request::GotoDeclarationParams;

use crate::Backend;
use crate::config_manager::ConfigSchemaStore;
use crate::handler::hover::get_hover_keys_with_span;

pub async fn handle_goto_declaration(
    backend: &Backend,
    params: GotoDeclarationParams,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    log::trace!("{:?}", params);

    let GotoDeclarationParams {
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
        .and_then(|server| server.goto_declaration.as_ref())
        .and_then(|goto_declaration| goto_declaration.enabled)
        .unwrap_or_default()
        .value()
    {
        log::debug!("`server.goto_declaration.enabled` is false");
        return Ok(None);
    }

    log::info!("handle_goto_declaration");

    let Some(document_source) = backend.document_source(&text_document_uri) else {
        return Ok(None);
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

    let Some((keys, _)) =
        get_hover_keys_with_span(&root, document_source.decoded(), offset, toml_version).await
    else {
        return Ok(None);
    };

    let document_tree = document_source.document_tree();
    let accessors = tombi_document_tree_syntax::get_accessors(document_tree, &keys, offset);

    if config.cargo_extension_enabled()
        && let Some(locations) = tombi_extension_cargo::goto_declaration(
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
        && let Some(locations) = tombi_extension_nagi_sql::goto_declaration(
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
        && let Some(locations) = tombi_extension_pyproject::goto_declaration(
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

    Ok(None)
}
