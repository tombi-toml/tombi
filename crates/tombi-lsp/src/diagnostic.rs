use std::path::PathBuf;

use ahash::AHashMap;
use itertools::{Either, Itertools};
use tombi_config::{Config, ConfigLevel, LintOptions};
use tombi_glob::{matches_file_patterns, MatchResult};

use crate::{backend::Backend, config_manager::ConfigSchemaStore, document::DocumentSource};

pub async fn publish_diagnostics(
    backend: &Backend,
    text_document_uri: tombi_uri::Uri,
    version: Option<i32>,
) {
    let Some(DiagnosticsResult {
        diagnostics,
        version: old_version,
    }) = get_diagnostics_result(backend, &text_document_uri).await
    else {
        return;
    };

    backend
        .client
        .publish_diagnostics(
            text_document_uri.into(),
            diagnostics,
            version.or(old_version),
        )
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

    let root = backend.get_incomplete_ast(text_document_uri).await?;

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(text_document_uri)))
        .await
        .ok()
        .flatten();

    let tombi_document_comment_directive =
        tombi_validator::comment_directive::get_tombi_document_comment_directive(&root).await;
    let (toml_version, _) = backend
        .source_toml_version(
            tombi_document_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let document_sources = backend.document_sources.read().await;

    match document_sources.get(text_document_uri) {
        Some(document) => Some(DiagnosticsResult {
            diagnostics: match tombi_linter::Linter::new(
                toml_version,
                config.lint.as_ref().unwrap_or(&LintOptions::default()),
                Some(Either::Left(text_document_uri)),
                &schema_store,
            )
            .lint(&document.text)
            .await
            {
                Ok(_) => Vec::with_capacity(0),
                Err(diagnostics) => diagnostics.into_iter().unique().map(Into::into).collect(),
            },
            version: document.version,
        }),
        None => None,
    }
}

#[derive(Debug)]
pub struct WorkspaceConfig {
    pub workspace_folder_path: PathBuf,
    pub config: Config,
    pub config_path: Option<PathBuf>,
    pub config_level: ConfigLevel,
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
        if let Ok((config, config_path, config_level)) =
            serde_tombi::config::load_with_path_and_level(Some(workspace_folder_path.clone()))
        {
            configs
                .entry(config_path.clone())
                .or_insert(WorkspaceConfig {
                    workspace_folder_path,
                    config,
                    config_path,
                    config_level,
                });
        };
    }

    Some(configs)
}

#[derive(Debug)]
pub struct WorkspaceDiagnosticTarget {
    pub text_document_uri: tombi_uri::Uri,
    pub version: Option<i32>,
}

pub async fn get_workspace_diagnostic_targets(
    backend: &Backend,
    workspace_config: &WorkspaceConfig,
) -> Option<Vec<WorkspaceDiagnosticTarget>> {
    let mut total_diagnostic_targets = Vec::new();

    let WorkspaceConfig {
        workspace_folder_path,
        config,
        config_level,
        config_path,
    } = workspace_config;

    let workspace_folder_path_str = workspace_folder_path.to_str()?;
    if let tombi_glob::FileSearch::Files(files) = tombi_glob::FileSearch::new(
        &[workspace_folder_path_str],
        config,
        config_path.as_deref(),
        *config_level,
    )
    .await
    {
        tracing::debug!(
            "Found {} files in {}: {:?}",
            files.len(),
            workspace_folder_path_str,
            files
        );

        for file in files {
            let Ok(text_document_path) = file else {
                continue;
            };
            if let Ok(text_document_uri) = tombi_uri::Uri::from_file_path(&text_document_path) {
                let Ok(content) = tokio::fs::read_to_string(&text_document_path).await else {
                    continue;
                };
                let version = {
                    backend
                        .document_sources
                        .write()
                        .await
                        .entry(text_document_uri.clone())
                        .or_insert_with(|| DocumentSource::new(content, None))
                        .version
                };

                total_diagnostic_targets.push(WorkspaceDiagnosticTarget {
                    text_document_uri,
                    version,
                });
            }
        }
    }

    if total_diagnostic_targets.is_empty() {
        None
    } else {
        Some(total_diagnostic_targets)
    }
}
