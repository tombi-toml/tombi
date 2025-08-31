use tower_lsp::lsp_types::{
    FullDocumentDiagnosticReport, WorkspaceDiagnosticParams, WorkspaceDiagnosticReport,
    WorkspaceDiagnosticReportResult, WorkspaceDocumentDiagnosticReport,
    WorkspaceFullDocumentDiagnosticReport,
};

use crate::{
    backend::{Backend, DiagnosticType},
    diagnostic::{
        get_diagnostics_result, get_workspace_configs, get_workspace_diagnostic_targets,
        publish_diagnostics, WorkspaceDiagnosticTarget,
    },
};

pub async fn handle_workspace_diagnostic(
    backend: &Backend,
    params: WorkspaceDiagnosticParams,
) -> Result<WorkspaceDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
    tracing::trace!("handle_workspace_diagnostic");
    tracing::trace!(?params);

    if let Some(configs) = get_workspace_configs(backend).await {
        let mut items = Vec::new();
        for workspace_config in configs.values() {
            if !workspace_config
                .config
                .lsp()
                .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
                .and_then(|workspace_diagnostic| workspace_diagnostic.enabled)
                .unwrap_or_default()
                .value()
            {
                tracing::debug!(
                    "`lsp.workspace-diagnostic.enabled` is false in {}",
                    workspace_config.workspace_folder_path.display()
                );
                continue;
            }

            if let Some(workspace_diagnostic_targets) =
                get_workspace_diagnostic_targets(backend, workspace_config).await
            {
                for WorkspaceDiagnosticTarget {
                    text_document_uri,
                    version,
                } in workspace_diagnostic_targets
                {
                    if let Some(diagnostics) =
                        get_diagnostics_result(backend, &text_document_uri).await
                    {
                        items.push(WorkspaceDocumentDiagnosticReport::Full(
                            WorkspaceFullDocumentDiagnosticReport {
                                uri: text_document_uri.into(),
                                version: version.map(|version| version as i64),
                                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                                    items: diagnostics.diagnostics,
                                    ..Default::default()
                                },
                            },
                        ));
                    }
                }
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

    if let Some(configs) = get_workspace_configs(backend).await {
        for workspace_config in configs.values() {
            if !workspace_config
                .config
                .lsp()
                .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
                .and_then(|workspace_diagnostic| workspace_diagnostic.enabled)
                .unwrap_or_default()
                .value()
            {
                tracing::debug!(
                    "`lsp.workspace-diagnostic.enabled` is false in {}",
                    workspace_config.workspace_folder_path.display()
                );
                continue;
            }
            if let Some(workspace_diagnostic_targets) =
                get_workspace_diagnostic_targets(backend, workspace_config).await
            {
                for WorkspaceDiagnosticTarget {
                    text_document_uri,
                    version,
                } in workspace_diagnostic_targets
                {
                    publish_diagnostics(backend, text_document_uri, version).await;
                }
            }
        }
    }
}
