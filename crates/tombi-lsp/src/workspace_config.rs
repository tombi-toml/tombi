use std::path::PathBuf;

use itertools::Itertools;
use tombi_config::Config;
use tombi_glob::{MatchResult, matches_file_patterns};

use crate::Backend;

#[derive(Debug)]
pub struct WorkspaceConfig {
    pub workspace_folder_path: PathBuf,
    pub config: Config,
    pub config_path: Option<PathBuf>,
}

impl WorkspaceConfig {
    #[inline]
    pub fn is_workspace_diagnostic_enabled(&self) -> bool {
        self.config
            .lsp
            .as_ref()
            .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
            .and_then(|workspace_diagnostic| workspace_diagnostic.enabled)
            .unwrap_or_default()
            .value()
    }

    #[inline]
    pub fn is_workspace_target(
        &self,
        text_document_path: &std::path::Path,
        home_dir: Option<&std::path::Path>,
    ) -> bool {
        if !self.is_workspace_diagnostic_enabled() {
            return false;
        }

        if let Some(home_dir) = home_dir
            && self.workspace_folder_path == home_dir
        {
            return false;
        }

        matches_file_patterns(
            text_document_path,
            self.config_path.as_deref(),
            &self.config,
        ) == MatchResult::Matched
    }
}

pub async fn get_workspace_configs(backend: &Backend) -> Option<Vec<WorkspaceConfig>> {
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

    log::debug!("workspace_folder_paths: {:?}", workspace_folder_paths);

    let workspace_folder_paths = workspace_folder_paths?;

    let mut configs = Vec::with_capacity(workspace_folder_paths.len());

    for workspace_folder_path in workspace_folder_paths {
        if let Ok((config, config_path)) =
            serde_tombi::config::load_with_path(Some(workspace_folder_path.clone()))
        {
            configs.push(WorkspaceConfig {
                workspace_folder_path,
                config,
                config_path,
            });
        };
    }

    Some(configs)
}

pub fn is_workspace_target(
    text_document_uri: &tombi_uri::Uri,
    workspace_configs: &[WorkspaceConfig],
    home_dir: Option<&std::path::Path>,
) -> bool {
    let Ok(text_document_path) = tombi_uri::Uri::to_file_path(text_document_uri) else {
        return false;
    };

    workspace_configs
        .iter()
        .any(|workspace_config| workspace_config.is_workspace_target(&text_document_path, home_dir))
}
