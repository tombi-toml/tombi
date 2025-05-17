use itertools::{Either, Itertools};
use tombi_config::LintOptions;
use tower_lsp::lsp_types::{
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    FullDocumentDiagnosticReport, RelatedFullDocumentDiagnosticReport,
};

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_diagnostic(
    backend: &Backend,
    params: DocumentDiagnosticParams,
) -> Result<DocumentDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_diagnostic");
    tracing::trace!(?params);

    let DocumentDiagnosticParams { text_document, .. } = params;

    let config = backend.config().await;

    if !config
        .lsp()
        .and_then(|server| server.diagnostics.as_ref())
        .and_then(|diagnostics| diagnostics.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.diagnostics.enabled` is false");
        return Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(Default::default()),
        ));
    }

    let Some(root) = backend.get_incomplete_ast(&text_document.uri).await else {
        return Ok(DocumentDiagnosticReportResult::Report(
            DocumentDiagnosticReport::Full(Default::default()),
        ));
    };

    let source_schema = backend
        .schema_store
        .build_source_schema_from_ast(&root, Some(Either::Left(&text_document.uri)))
        .await
        .ok()
        .flatten();

    let (toml_version, _) = backend.source_toml_version(source_schema.as_ref()).await;

    let document_sources = backend.document_sources.read().await;

    let diagnostics = match document_sources.get(&text_document.uri) {
        Some(document) => tombi_linter::Linter::new(
            toml_version,
            backend
                .config()
                .await
                .lint
                .as_ref()
                .unwrap_or(&LintOptions::default()),
            Some(Either::Left(&text_document.uri)),
            &backend.schema_store,
        )
        .lint(&document.source)
        .await
        .map_or_else(
            |diagnostics| diagnostics.into_iter().unique().map(Into::into).collect(),
            |_| vec![],
        ),
        None => vec![],
    };

    Ok(DocumentDiagnosticReportResult::Report(
        DocumentDiagnosticReport::Full(RelatedFullDocumentDiagnosticReport {
            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                items: diagnostics,
                ..Default::default()
            },
            ..Default::default()
        }),
    ))
}
