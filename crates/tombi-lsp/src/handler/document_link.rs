use tombi_ast::SchemaDocumentCommentDirective;
use tombi_extension::get_tombi_github_uri;
use tombi_text::IntoLsp;
use tower_lsp::lsp_types::{DocumentLink, DocumentLinkParams};

use crate::{config_manager::ConfigSchemaStore, Backend};

pub async fn handle_document_link(
    backend: &Backend,
    params: DocumentLinkParams,
) -> Result<Option<Vec<DocumentLink>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_document_link");
    tracing::trace!(?params);

    let DocumentLinkParams { text_document, .. } = params;
    let text_document_uri = text_document.uri.into();

    let ConfigSchemaStore { config, .. } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    if !config
        .lsp()
        .and_then(|server| server.document_link.as_ref())
        .and_then(|document_link| document_link.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.document_link.enabled` is false");
        return Ok(None);
    }

    let document_sources = backend.document_sources.read().await;
    let Some(document_source) = document_sources.get(&text_document_uri) else {
        return Ok(None);
    };

    let root = document_source.ast();
    let toml_version = document_source.toml_version;
    let line_index = document_source.line_index();

    let mut document_links = vec![];

    if let Some(SchemaDocumentCommentDirective {
        uri: Ok(schema_uri),
        uri_range: range,
        ..
    }) = root.schema_document_comment_directive(text_document_uri.to_file_path().ok().as_deref())
    {
        let tooltip = "Open JSON Schema".into();
        document_links.push(
            tombi_extension::DocumentLink {
                range,
                target: get_tombi_github_uri(&schema_uri).unwrap_or(schema_uri.into()),
                tooltip,
            }
            .into_lsp(line_index),
        );
    }

    let document_tree = document_source.document_tree();

    if let Some(locations) =
        tombi_extension_cargo::document_link(&text_document_uri, document_tree, toml_version)
            .await?
    {
        document_links.extend(
            locations
                .into_iter()
                .map(|location| location.into_lsp(line_index)),
        );
    }

    if let Some(locations) =
        tombi_extension_tombi::document_link(&text_document_uri, document_tree, toml_version)
            .await?
    {
        document_links.extend(
            locations
                .into_iter()
                .map(|location| location.into_lsp(line_index)),
        );
    }

    if let Some(locations) =
        tombi_extension_uv::document_link(&text_document_uri, document_tree, toml_version).await?
    {
        document_links.extend(
            locations
                .into_iter()
                .map(|location| location.into_lsp(line_index)),
        );
    }

    Ok(Some(document_links))
}
