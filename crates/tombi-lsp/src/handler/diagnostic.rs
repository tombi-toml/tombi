use tower_lsp::lsp_types::{
    DocumentDiagnosticParams, DocumentDiagnosticReport, DocumentDiagnosticReportResult,
    FullDocumentDiagnosticReport, RelatedFullDocumentDiagnosticReport,
};

use crate::{backend::Backend, diagnostic::get_diagnostics_result};

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
pub async fn push_diagnostics(backend: &Backend, text_document_uri: tombi_uri::Uri) {
    if !backend.is_diagnostic_mode_push().await {
        return;
    }

    #[derive(Debug)]
    struct PushDiagnosticsParams {
        text_document: TextDocumentIdentifier,
    }

    #[derive(Debug)]
    struct TextDocumentIdentifier {
        uri: tombi_uri::Uri,
    }

    let params = PushDiagnosticsParams {
        text_document: TextDocumentIdentifier {
            uri: text_document_uri,
        },
    };

    tracing::info!("push_diagnostics");
    tracing::trace!("{:?}", params);

    let Some(diagnostics_result) = get_diagnostics_result(backend, &params.text_document.uri).await
    else {
        return;
    };

    tracing::trace!("{:?}", diagnostics_result);

    let crate::diagnostic::DiagnosticsResult {
        diagnostics,
        version,
    } = diagnostics_result;

    backend
        .client
        .publish_diagnostics(params.text_document.uri.into(), diagnostics, version)
        .await;
}
