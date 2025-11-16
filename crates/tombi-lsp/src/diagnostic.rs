use std::path::PathBuf;

use ahash::AHashMap;
use itertools::{Either, Itertools};
use tombi_config::{Config, LintOptions};
use tombi_glob::{matches_file_patterns, MatchResult};
use tombi_text::IntoLsp;

use crate::{backend::Backend, config_manager::ConfigSchemaStore};

pub async fn publish_diagnostics(backend: &Backend, text_document_uri: tombi_uri::Uri) {
    let Some(diagnostics_result) = get_diagnostics_result(backend, &text_document_uri).await else {
        return;
    };

    tracing::trace!(?diagnostics_result);

    let DiagnosticsResult {
        diagnostics,
        version,
    } = diagnostics_result;

    backend
        .client
        .publish_diagnostics(text_document_uri.into(), diagnostics, version)
        .await
}

#[derive(Debug)]
pub struct DiagnosticsResult {
    pub diagnostics: Vec<tower_lsp::lsp_types::Diagnostic>,
    pub version: Option<i32>,
}

pub async fn get_diagnostics_result(
    backend: &Backend,
    text_document_uri: &tombi_uri::Uri,
) -> Option<DiagnosticsResult> {
    let ConfigSchemaStore {
        config,
        schema_store,
        config_path,
    } = backend
        .config_manager
        .config_schema_store_for_uri(text_document_uri)
        .await;

    if !config
        .lsp()
        .and_then(|lsp| lsp.diagnostic())
        .and_then(|diagnostic| diagnostic.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`lsp.diagnostic.enabled` is false");
        return None;
    }

    if let Ok(text_document_path) = tombi_uri::Uri::to_file_path(text_document_uri) {
        match matches_file_patterns(&text_document_path, config_path.as_deref(), &config) {
            MatchResult::Matched => {}
            MatchResult::IncludeNotMatched => {
                tracing::info!(
                    "Skip {text_document_path:?} because it is not in config.files.include"
                );
                return None;
            }
            MatchResult::ExcludeMatched => {
                tracing::info!("Skip {text_document_path:?} because it is in config.files.exclude");
                return None;
            }
        }
    }

    let document_sources = backend.document_sources.read().await;

    match document_sources.get(text_document_uri) {
        Some(document_source) => Some(DiagnosticsResult {
            diagnostics: match tombi_linter::Linter::new(
                document_source.toml_version,
                config.lint.as_ref().unwrap_or(&LintOptions::default()),
                Some(Either::Left(text_document_uri)),
                &schema_store,
            )
            .lint(document_source.text())
            .await
            {
                Ok(_) => Vec::with_capacity(0),
                Err(diagnostics) => diagnostics
                    .into_iter()
                    .unique()
                    .map(|diagnostic| diagnostic.into_lsp(document_source.line_index()))
                    .collect_vec(),
            },
            version: document_source.version,
        }),
        None => None,
    }
}

#[derive(Debug)]
pub struct WorkspaceConfig {
    pub workspace_folder_path: PathBuf,
    pub config: Config,
}

pub async fn get_workspace_configs(
    backend: &Backend,
) -> Option<AHashMap<Option<PathBuf>, WorkspaceConfig>> {
    let workspace_folder_paths =
        backend
            .client
            .workspace_folders()
            .await
            .ok()
            .flatten()
            .map(|workspace_folders| {
                workspace_folders
                    .into_iter()
                    .filter_map(|workspace| {
                        tombi_uri::Uri::to_file_path(&workspace.uri.into()).ok()
                    })
                    .collect_vec()
            });

    tracing::debug!("workspace_folder_paths: {:?}", workspace_folder_paths);

    let workspace_folder_paths = workspace_folder_paths?;

    let mut configs = AHashMap::new();

    for workspace_folder_path in workspace_folder_paths {
        if let Ok((config, config_path)) =
            serde_tombi::config::load_with_path(Some(workspace_folder_path.clone()))
        {
            configs
                .entry(config_path.clone())
                .or_insert(WorkspaceConfig {
                    workspace_folder_path,
                    config,
                });
        };
    }

    Some(configs)
}
