use tower_lsp::lsp_types::{
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    FullDocumentDiagnosticReport, RelatedFullDocumentDiagnosticReport, TextDocumentIdentifier,
};

use crate::{
    backend::{Backend, DiagnosticType},
    diagnostic::{get_diagnostics_result, publish_diagnostics},
};

/// Pull diagnostics
pub async fn handle_diagnostic(
    backend: &Backend,
    params: DocumentDiagnosticParams,
) -> Result<DocumentDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
    let DocumentDiagnosticParams { text_document, .. } = params;

    let text_document_uri = text_document.uri.into();

    Ok({
        DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(
            RelatedFullDocumentDiagnosticReport {
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    items: get_diagnostics_result(backend, &text_document_uri)
                        .await
                        .map(|result| result.diagnostics)
                        .unwrap_or_default(),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
    })
}

/// Push diagnostics
pub async fn push_diagnostics(
    backend: &Backend,
    text_document_uri: tombi_uri::Uri,
    version: Option<i32>,
) {
    if backend.capabilities.read().await.diagnostic_type != DiagnosticType::Push {
        return;
    }

    #[derive(Debug)]
    struct PushDiagnosticsParams {
        text_document: TextDocumentIdentifier,
        version: Option<i32>,
    }

    let params = PushDiagnosticsParams {
        text_document: TextDocumentIdentifier {
            uri: text_document_uri.into(),
        },
        version,
    };

    tracing::info!("push_diagnostics");
    tracing::trace!(?params);

    publish_diagnostics(backend, params.text_document.uri.into(), params.version).await;
}
