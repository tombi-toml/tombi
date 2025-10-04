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
    tracing::info!("handle_workspace_diagnostic");
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
                let mut skipped_count = 0;
                for WorkspaceDiagnosticTarget {
                    text_document_uri,
                    version,
                    should_skip,
                } in workspace_diagnostic_targets
                {
                    // Skip processing if should_skip flag is true
                    if should_skip {
                        skipped_count += 1;
                        continue;
                    }

                    if let Some(diagnostics) =
                        get_diagnostics_result(backend, &text_document_uri).await
                    {
                        // Record mtime after diagnostic execution
                        if let Ok(path) = tombi_uri::Uri::to_file_path(&text_document_uri) {
                            if let Ok(metadata) = tokio::fs::metadata(&path).await {
                                if let Ok(mtime) = metadata.modified() {
                                    backend
                                        .mtime_tracker
                                        .record(text_document_uri.clone(), mtime)
                                        .await;
                                    tracing::debug!("Recorded mtime for {:?}", path);
                                }
                            }
                        }

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
                tracing::debug!("Skipped {} files in workspace diagnostics", skipped_count);
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
                let mut skipped_count = 0;
                for WorkspaceDiagnosticTarget {
                    text_document_uri,
                    version,
                    should_skip,
                } in workspace_diagnostic_targets
                {
                    // Skip processing if should_skip flag is true
                    if should_skip {
                        skipped_count += 1;
                        continue;
                    }

                    publish_diagnostics(backend, text_document_uri.clone(), version).await;

                    // Record mtime after diagnostic execution
                    if let Ok(path) = tombi_uri::Uri::to_file_path(&text_document_uri) {
                        if let Ok(metadata) = tokio::fs::metadata(&path).await {
                            if let Ok(mtime) = metadata.modified() {
                                backend.mtime_tracker.record(text_document_uri, mtime).await;
                                tracing::debug!("Recorded mtime for {:?}", path);
                            }
                        }
                    }
                }
                tracing::debug!(
                    "Skipped {} files in push workspace diagnostics",
                    skipped_count
                );
            }
        }
    }
}
