use tombi_ast_syntax::SchemaDocumentCommentDirective;
use tombi_extension::get_tombi_github_uri;
use tower_lsp::lsp_types::{DocumentLink, DocumentLinkParams};

use crate::extension::IntoLsp as _;
use crate::{Backend, config_manager::ConfigSchemaStore};

pub async fn handle_document_link(
    backend: &Backend,
    params: DocumentLinkParams,
) -> Result<Option<Vec<DocumentLink>>, tower_lsp::jsonrpc::Error> {
    log::trace!("{:?}", params);

    let DocumentLinkParams { text_document, .. } = params;
    let text_document_uri = text_document.uri.into();

    let ConfigSchemaStore { config, .. } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    if !config
        .lsp
        .as_ref()
        .and_then(|server| server.document_link.as_ref())
        .and_then(|document_link| document_link.enabled)
        .unwrap_or_default()
        .value()
    {
        log::debug!("`server.document_link.enabled` is false");
        return Ok(None);
    }

    log::info!("handle_document_link");

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

    let mut document_links = vec![];

    if let Some(SchemaDocumentCommentDirective {
        uri: Ok(schema_uri),
        uri_span: span,
        ..
    }) = root.schema_document_comment_directive(text_document_uri.to_file_path().ok().as_deref())
    {
        let tooltip = "Open JSON Schema".into();
        document_links.push(
            tombi_extension::DocumentLink {
                span,
                target: get_tombi_github_uri(&schema_uri).unwrap_or(schema_uri.into()),
                tooltip,
            }
            .into_lsp_type(line_index, encoding),
        );
    }

    let document_tree = document_source.document_tree();

    if config.cargo_extension_enabled()
        && let Some(locations) = tombi_extension_cargo::document_link(
            &text_document_uri,
            document_tree,
            toml_version,
            converter,
            config.cargo_extension_features(),
        )
        .await?
    {
        document_links.extend(
            locations
                .into_iter()
                .map(|location| location.into_lsp_type(line_index, encoding)),
        );
    }

    if config.tombi_extension_enabled()
        && let Some(locations) = tombi_extension_tombi::document_link(
            &text_document_uri,
            document_tree,
            toml_version,
            config.tombi_extension_features(),
        )
        .await?
    {
        document_links.extend(
            locations
                .into_iter()
                .map(|location| location.into_lsp_type(line_index, encoding)),
        );
    }

    if config.pyproject_extension_enabled()
        && let Some(locations) = tombi_extension_pyproject::document_link(
            &text_document_uri,
            document_tree,
            toml_version,
            config.pyproject_extension_features(),
        )
        .await?
    {
        document_links.extend(
            locations
                .into_iter()
                .map(|location| location.into_lsp_type(line_index, encoding)),
        );
    }

    if document_links.is_empty() {
        return Ok(None);
    }

    Ok(Some(document_links))
}
