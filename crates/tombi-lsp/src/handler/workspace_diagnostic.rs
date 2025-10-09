use tower_lsp::lsp_types::{
    FullDocumentDiagnosticReport, WorkspaceDiagnosticParams, WorkspaceDiagnosticReport,
    WorkspaceDiagnosticReportResult, WorkspaceDocumentDiagnosticReport,
    WorkspaceFullDocumentDiagnosticReport,
};

use crate::{
    backend::{Backend, DiagnosticType},
    diagnostic::{
        get_diagnostics_result, get_workspace_configs, get_workspace_diagnostic_targets,
        publish_diagnostics, WorkspaceConfig, WorkspaceDiagnosticTarget,
    },
};

pub async fn handle_workspace_diagnostic(
    backend: &Backend,
    params: WorkspaceDiagnosticParams,
) -> Result<WorkspaceDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_workspace_diagnostic");
    tracing::trace!(?params);

    let items = execute_workspace_diagnostics(backend, false)
        .await
        .unwrap_or_default();

    Ok(WorkspaceDiagnosticReportResult::Report(
        WorkspaceDiagnosticReport { items },
    ))
}

pub async fn push_workspace_diagnostics(backend: &Backend) {
    if backend.capabilities.read().await.diagnostic_type != DiagnosticType::Push {
        return;
    }

    tracing::info!("push_workspace_diagnostics");

    let _ = execute_workspace_diagnostics(backend, true).await;
}

/// Execute workspace diagnostics with common logic
async fn execute_workspace_diagnostics(
    backend: &Backend,
    is_push_mode: bool,
) -> Option<Vec<WorkspaceDocumentDiagnosticReport>> {
    let configs = get_workspace_configs(backend).await?;
    let mut all_items = Vec::new();
    let home_dir = dirs::home_dir();

    for workspace_config in configs.values() {
        // Check if workspace diagnostic is enabled first (priority over throttle)
        if !is_workspace_diagnostic_enabled(workspace_config) {
            tracing::debug!(
                "`lsp.workspace-diagnostic.enabled` is false in {}",
                workspace_config.workspace_folder_path.display()
            );
            continue;
        }

        if let Some(home_dir) = &home_dir {
            if &workspace_config.workspace_folder_path == home_dir {
                tracing::debug!(
                    "Skip diagnostics for workspace folder matching $HOME: {:?}",
                    workspace_config.workspace_folder_path
                );
                continue;
            }
        }

        // Check throttling only if enabled
        if let Some(throttle_seconds) = get_throttle_seconds(workspace_config) {
            match backend
                .workspace_diagnostic_state
                .throttle()
                .should_skip_by_throttle(throttle_seconds)
                .await
            {
                Ok((should_skip, elapsed_secs)) => {
                    if should_skip {
                        if let Some(elapsed) = elapsed_secs {
                            if elapsed == 0.0 {
                                tracing::debug!("Workspace diagnostics skipped by `workspace-diagnostic.throttle-seconds = 0`, workspace_folder_path={}, config_path={:?}",
                                    workspace_config.workspace_folder_path.display(),
                                    workspace_config.config_path,
                                );
                            } else {
                                tracing::debug!(
                                    "Workspace diagnostics skipped by throttle: elapsed {:.2}s < {}s, workspace_folder_path={}, config_path={:?}",
                                    elapsed,
                                    throttle_seconds,
                                    workspace_config.workspace_folder_path.display(),
                                    workspace_config.config_path,
                                );
                            }
                        }
                        continue;
                    } else if let Some(elapsed) = elapsed_secs {
                        tracing::debug!(
                            "Workspace diagnostics executing: elapsed {:.2}s >= {}s, workspace_folder_path={}, config_path={:?}",
                            elapsed,
                            throttle_seconds,
                            workspace_config.workspace_folder_path.display(),
                            workspace_config.config_path,
                        );
                    }
                }
                Err(err) => {
                    tracing::error!(
                        "Failed to check workspace diagnostics throttle: {}, proceeding without throttle, workspace_folder_path={}, config_path={:?}",
                        err,
                        workspace_config.workspace_folder_path.display(),
                        workspace_config.config_path,
                    );
                }
            }
        }

        let (items, skipped_count) =
            process_workspace_diagnostic_targets(backend, workspace_config, is_push_mode).await;

        all_items.extend(items);
        tracing::debug!(
            "Skipped {} files in workspace diagnostics, workspace_folder_path={}, config_path={:?}",
            skipped_count,
            workspace_config.workspace_folder_path.display(),
            workspace_config.config_path
        );
    }

    // Record completion time
    backend
        .workspace_diagnostic_state
        .throttle()
        .record_completion()
        .await;

    Some(all_items)
}

/// Check if workspace diagnostic is enabled for the given workspace config
fn is_workspace_diagnostic_enabled(workspace_config: &WorkspaceConfig) -> bool {
    workspace_config
        .config
        .lsp()
        .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
        .and_then(|workspace_diagnostic| workspace_diagnostic.enabled)
        .unwrap_or_default()
        .value()
}

/// Get throttle seconds from workspace config
fn get_throttle_seconds(workspace_config: &WorkspaceConfig) -> Option<u64> {
    workspace_config
        .config
        .lsp()
        .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
        .and_then(|wd| wd.throttle_seconds)
}

/// Process workspace diagnostic targets for a single workspace
async fn process_workspace_diagnostic_targets(
    backend: &Backend,
    workspace_config: &WorkspaceConfig,
    is_push_mode: bool,
) -> (Vec<WorkspaceDocumentDiagnosticReport>, usize) {
    let mut items = Vec::new();
    let mut skipped_count = 0;

    if let Some(workspace_diagnostic_targets) =
        get_workspace_diagnostic_targets(backend, workspace_config).await
    {
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

            // Clone URI for mtime recording
            let text_document_uri_clone = text_document_uri.clone();

            if is_push_mode {
                publish_diagnostics(backend, text_document_uri, version).await;
            } else if let Some(diagnostics) =
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

            // Record mtime after diagnostic execution
            if let Ok(path) = tombi_uri::Uri::to_file_path(&text_document_uri_clone) {
                if let Ok(metadata) = tokio::fs::metadata(&path).await {
                    if let Ok(mtime) = metadata.modified() {
                        backend
                            .workspace_diagnostic_state
                            .mtime_tracker()
                            .record(text_document_uri_clone, mtime)
                            .await;
                        tracing::debug!("Recorded mtime for {:?}", path);
                    }
                }
            }
        }
    }

    (items, skipped_count)
}
