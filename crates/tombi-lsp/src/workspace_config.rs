use std::path::PathBuf;

use itertools::Itertools;
use tombi_config::Config;

use crate::Backend;

#[derive(Debug)]
pub struct WorkspaceConfig {
    pub workspace_folder_path: PathBuf,
    pub config: Config,
}

pub async fn get_workspace_configs(
    backend: &Backend,
) -> Option<tombi_hashmap::HashMap<Option<PathBuf>, WorkspaceConfig>> {
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

    let mut configs = tombi_hashmap::HashMap::new();

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
