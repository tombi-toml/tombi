use tower_lsp_server::ls_types::lsp::{
    FullDocumentDiagnosticReport, WorkspaceDiagnosticParams, WorkspaceDiagnosticReport,
    WorkspaceDiagnosticReportResult, WorkspaceDocumentDiagnosticReport,
    WorkspaceFullDocumentDiagnosticReport,
};

use crate::{
    backend::{Backend, DiagnosticType},
    diagnostic::{
        get_diagnostics_result, get_workspace_diagnostic_targets, publish_diagnostics,
        WorkspaceDiagnosticTarget,
    },
};

pub async fn handle_workspace_diagnostic(
    backend: &Backend,
    params: WorkspaceDiagnosticParams,
) -> Result<WorkspaceDiagnosticReportResult, tower_lsp_server::jsonrpc::Error> {
    tracing::info!("handle_workspace_diagnostic");
    tracing::trace!(?params);

    if let Some(workspace_diagnostic_targets) = get_workspace_diagnostic_targets(backend).await {
        let mut items = Vec::new();
        for WorkspaceDiagnosticTarget { uri, version } in workspace_diagnostic_targets {
            if let Some(diagnostics) = get_diagnostics_result(backend, &uri).await {
                items.push(WorkspaceDocumentDiagnosticReport::Full(
                    WorkspaceFullDocumentDiagnosticReport {
                        uri: uri.into(),
                        version: version.map(|version| version as i64),
                        full_document_diagnostic_report: FullDocumentDiagnosticReport {
                            items: diagnostics.diagnostics,
                            ..Default::default()
                        },
                    },
                ));
            }
        }

        return Ok(WorkspaceDiagnosticReportResult::Report(
            WorkspaceDiagnosticReport { items },
        ));
    }

    Ok(WorkspaceDiagnosticReportResult::Report(
        WorkspaceDiagnosticReport { items: vec![] },
    ))
}

pub async fn push_workspace_diagnostics(backend: &Backend) {
    if backend.capabilities.read().await.diagnostic_type != DiagnosticType::Push {
        return;
    }

    tracing::info!("push_workspace_diagnostics");

    if let Some(workspace_diagnostic_targets) = get_workspace_diagnostic_targets(backend).await {
        for WorkspaceDiagnosticTarget { uri, version } in workspace_diagnostic_targets {
            publish_diagnostics(backend, uri, version).await;
        }
    };
}
