use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use tower_lsp::lsp_types::{
    FullDocumentDiagnosticReport, UnchangedDocumentDiagnosticReport, WorkspaceDiagnosticParams,
    WorkspaceDiagnosticReport, WorkspaceDiagnosticReportResult, WorkspaceDocumentDiagnosticReport,
    WorkspaceFullDocumentDiagnosticReport, WorkspaceUnchangedDocumentDiagnosticReport,
};

use crate::{
    backend::Backend,
    diagnostic::{DiagnosticsResult, get_diagnostics_result},
    workspace_diagnostic::collect_workspace_diagnostic_targets,
};

pub async fn handle_workspace_diagnostic(
    backend: &Backend,
    params: WorkspaceDiagnosticParams,
) -> Result<WorkspaceDiagnosticReportResult, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_workspace_diagnostic");
    tracing::trace!("{:?}", params);

    let previous_result_ids = params
        .previous_result_ids
        .into_iter()
        .map(|result| (tombi_uri::Uri::from(result.uri), result.value))
        .collect::<tombi_hashmap::HashMap<_, _>>();

    let targets = collect_workspace_diagnostic_targets(backend).await;
    let target_set = targets
        .iter()
        .cloned()
        .collect::<tombi_hashmap::HashSet<_>>();
    let mut items = Vec::new();

    for text_document_uri in targets {
        let Some(diagnostics_result) = get_diagnostics_result(backend, &text_document_uri).await
        else {
            continue;
        };

        let DiagnosticsResult {
            diagnostics,
            version,
        } = diagnostics_result;
        let result_id = workspace_diagnostic_result_id(version, &diagnostics);

        if previous_result_ids
            .get(&text_document_uri)
            .is_some_and(|previous_result_id| previous_result_id == &result_id)
        {
            items.push(WorkspaceDocumentDiagnosticReport::Unchanged(
                WorkspaceUnchangedDocumentDiagnosticReport {
                    uri: text_document_uri.into(),
                    version: version.map(i64::from),
                    unchanged_document_diagnostic_report: UnchangedDocumentDiagnosticReport {
                        result_id,
                    },
                },
            ));
        } else {
            items.push(WorkspaceDocumentDiagnosticReport::Full(
                WorkspaceFullDocumentDiagnosticReport {
                    uri: text_document_uri.into(),
                    version: version.map(i64::from),
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: Some(result_id),
                        items: diagnostics,
                    },
                },
            ));
        }
    }

    for previous_text_document_uri in previous_result_ids.keys() {
        if target_set.contains(previous_text_document_uri) {
            continue;
        }

        items.push(WorkspaceDocumentDiagnosticReport::Full(
            WorkspaceFullDocumentDiagnosticReport {
                uri: previous_text_document_uri.clone().into(),
                version: None,
                full_document_diagnostic_report: FullDocumentDiagnosticReport {
                    result_id: Some(workspace_diagnostic_result_id(None, &[])),
                    items: Vec::new(),
                },
            },
        ));
    }

    Ok(WorkspaceDiagnosticReportResult::Report(
        WorkspaceDiagnosticReport { items },
    ))
}

pub fn workspace_diagnostic_result_id(
    version: Option<i32>,
    diagnostics: &[tower_lsp::lsp_types::Diagnostic],
) -> String {
    let mut hasher = DefaultHasher::new();
    version.hash(&mut hasher);
    serde_json::to_string(diagnostics)
        .unwrap_or_default()
        .hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
