use tombi_config::DEFAULT_THROTTLE_SECONDS;
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

    // Check throttling
    if let Some(configs) = get_workspace_configs(backend).await {
        // Get throttle_seconds from the first workspace config
        let throttle_seconds = configs
            .iter()
            .next()
            .and_then(|(config_path, config)| {
                if let Some(config_path) = config_path {
                    tracing::trace!("load throttle_seconds from {}", config_path.display());
                } else {
                    tracing::trace!("load throttle_seconds from default config");
                }

                config
                    .config
                    .lsp()
                    .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
                    .and_then(|wd| wd.throttle_seconds)
            })
            .unwrap_or_else(|| {
                tracing::trace!("use default throttle_seconds");
                DEFAULT_THROTTLE_SECONDS
            });

        match backend
            .workspace_diagnostics_throttle
            .should_skip_by_throttle(throttle_seconds)
            .await
        {
            Ok((should_skip, elapsed_secs)) => {
                if should_skip {
                    if let Some(elapsed) = elapsed_secs {
                        tracing::info!(
                            "Workspace diagnostics skipped by throttle: elapsed {:.2}s < {}s",
                            elapsed,
                            throttle_seconds
                        );
                    } else {
                        tracing::info!("Workspace diagnostics skipped by `workspace-diagnostic.throttle-seconds = 0`");
                    }
                    return Ok(WorkspaceDiagnosticReportResult::Report(
                        WorkspaceDiagnosticReport { items: vec![] },
                    ));
                } else if let Some(elapsed) = elapsed_secs {
                    tracing::debug!(
                        "Workspace diagnostics executing: elapsed {:.2}s >= {}s",
                        elapsed,
                        throttle_seconds
                    );
                }
            }
            Err(err) => {
                tracing::error!(
                    "Failed to check workspace diagnostics throttle: {}, proceeding without throttle",
                    err
                );
            }
        }
    }

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

        // Record completion time
        backend
            .workspace_diagnostics_throttle
            .record_completion()
            .await;
        tracing::debug!("Recorded workspace diagnostics completion time");

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

    // Check throttling
    if let Some(configs) = get_workspace_configs(backend).await {
        // Get throttle_seconds from the first workspace config
        let throttle_seconds = configs
            .values()
            .next()
            .and_then(|config| {
                config
                    .config
                    .lsp()
                    .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
                    .and_then(|wd| wd.throttle_seconds)
            })
            .unwrap_or(5); // Default: 5 seconds

        match backend
            .workspace_diagnostics_throttle
            .should_skip_by_throttle(throttle_seconds)
            .await
        {
            Ok((should_skip, elapsed_secs)) => {
                if should_skip {
                    let elapsed = elapsed_secs.unwrap_or(0.0);
                    tracing::info!(
                        "Push workspace diagnostics skipped by throttle: elapsed {:.2}s < {}s",
                        elapsed,
                        throttle_seconds
                    );
                    return;
                } else if let Some(elapsed) = elapsed_secs {
                    tracing::debug!(
                        "Push workspace diagnostics executing: elapsed {:.2}s >= {}s",
                        elapsed,
                        throttle_seconds
                    );
                }
            }
            Err(err) => {
                tracing::error!(
                    "Failed to check push workspace diagnostics throttle: {}, proceeding without throttle",
                    err
                );
            }
        }
    }

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

        // Record completion time
        backend
            .workspace_diagnostics_throttle
            .record_completion()
            .await;
        tracing::debug!("Recorded push workspace diagnostics completion time");
    }
}
