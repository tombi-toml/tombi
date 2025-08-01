use itertools::Either;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tower_lsp::lsp_types::{DocumentLink, DocumentLinkParams};

use crate::Backend;

pub async fn handle_document_link(
    backend: &Backend,
    params: DocumentLinkParams,
) -> Result<Option<Vec<DocumentLink>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_document_link");
    tracing::trace!(?params);

    let DocumentLinkParams { text_document, .. } = params;

    let config = backend.config().await;

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

    let Some(root) = backend.get_incomplete_ast(&text_document.uri).await else {
        return Ok(None);
    };

    let mut document_links = vec![];

    // Document Inline Comment Schema URL
    //
    // ```toml
    // #:schema https://example.com/schema.json
    // key = "value"
    // ```
    if let Some((Ok(schema_url), range)) =
        root.file_schema_url(text_document.uri.to_file_path().ok().as_deref())
    {
        let tooltip = "Open JSON Schema".into();
        document_links.push(
            tombi_extension::DocumentLink {
                range,
                target: schema_url,
                tooltip,
            }
            .into(),
        );
    }

    // Document Link for Extensions
    let source_schema = backend
        .schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document.uri)))
        .await
        .ok()
        .flatten();

    let (toml_version, _) = backend.source_toml_version(source_schema.as_ref()).await;

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;

    if let Some(locations) =
        tombi_extension_cargo::document_link(&text_document, &document_tree, toml_version).await?
    {
        document_links.extend(locations.into_iter().map(Into::into));
    }

    if let Some(locations) =
        tombi_extension_tombi::document_link(&text_document, &document_tree, toml_version).await?
    {
        document_links.extend(locations.into_iter().map(Into::into));
    }

    if let Some(locations) =
        tombi_extension_uv::document_link(&text_document, &document_tree, toml_version).await?
    {
        document_links.extend(locations.into_iter().map(Into::into));
    }

    Ok(Some(document_links))
}
