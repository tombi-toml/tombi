use itertools::{Either, Itertools};
use tombi_config::LintOptions;
use tower_lsp::lsp_types::{TextDocumentIdentifier, Url};

use crate::{backend::Backend, config_manager::ConfigSchemaStore};

pub async fn publish_diagnostics(backend: &Backend, text_document_uri: Url, version: Option<i32>) {
    #[derive(Debug)]
    struct PublishDiagnosticsParams {
        text_document: TextDocumentIdentifier,
        version: Option<i32>,
    }

    let params = PublishDiagnosticsParams {
        text_document: TextDocumentIdentifier {
            uri: text_document_uri,
        },
        version,
    };

    tracing::info!("publish_diagnostics");
    tracing::trace!(?params);

    let Some(diagnostics) = diagnostics(backend, &params.text_document.uri).await else {
        return;
    };

    backend
        .client
        .publish_diagnostics(params.text_document.uri, diagnostics, params.version)
        .await
}

async fn diagnostics(
    backend: &Backend,
    text_document_uri: &Url,
) -> Option<Vec<tower_lsp::lsp_types::Diagnostic>> {
    let ConfigSchemaStore {
        config,
        schema_store,
    } = backend
        .config_manager
        .config_schema_store_for_url(text_document_uri)
        .await;

    if !config
        .lsp()
        .and_then(|server| server.diagnostics.as_ref())
        .and_then(|diagnostics| diagnostics.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.diagnostics.enabled` is false");
        return None;
    }

    let root = backend.get_incomplete_ast(text_document_uri).await?;

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(text_document_uri)))
        .await
        .ok()
        .flatten();

    let root_comment_directive = tombi_comment_directive::get_tombi_comment_directive(&root).await;
    let (toml_version, _) = backend
        .source_toml_version(root_comment_directive, source_schema.as_ref(), &config)
        .await;

    let document_sources = backend.document_sources.read().await;

    match document_sources.get(text_document_uri) {
        Some(document) => tombi_linter::Linter::new(
            toml_version,
            config.lint.as_ref().unwrap_or(&LintOptions::default()),
            Some(Either::Left(text_document_uri)),
            &schema_store,
        )
        .lint(&document.text)
        .await
        .map_or_else(
            |diagnostics| Some(diagnostics.into_iter().unique().map(Into::into).collect()),
            |_| Some(Vec::with_capacity(0)),
        ),
        None => None,
    }
}
