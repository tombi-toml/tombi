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
        is_workspace_diagnostic_enabled(&self.config)
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

    #[inline]
    fn is_ignored(&self, text_document_path: &std::path::Path) -> bool {
        self.config
            .files
            .as_ref()
            .is_none_or(|files| files.respect_ignore_files.value())
            && tombi_glob::is_path_ignored(&self.workspace_folder_path, text_document_path)
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

/// Whether the workspace diagnostic feature is enabled for the given config.
///
/// Workspace diagnostics are computed through the same pipeline as document diagnostics,
/// so `lsp.diagnostic.enabled = false` disables them as well.
/// Without this, the whole workspace would still be crawled and linted only to
/// throw the results away.
#[inline]
pub fn is_workspace_diagnostic_enabled(config: &Config) -> bool {
    if !is_diagnostic_enabled(config) {
        return false;
    }

    config
        .lsp
        .as_ref()
        .and_then(|lsp| lsp.workspace_diagnostic.as_ref())
        .and_then(|workspace_diagnostic| workspace_diagnostic.enabled)
        .unwrap_or_default()
        .value()
}

/// Whether the document diagnostic feature is enabled for the given config.
#[inline]
pub fn is_diagnostic_enabled(config: &Config) -> bool {
    config
        .lsp
        .as_ref()
        .and_then(|lsp| lsp.diagnostic.as_ref())
        .and_then(|diagnostic| diagnostic.enabled)
        .unwrap_or_default()
        .value()
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

pub fn is_workspace_ignored(
    text_document_uri: &tombi_uri::Uri,
    workspace_configs: &[WorkspaceConfig],
) -> bool {
    let Ok(text_document_path) = tombi_uri::Uri::to_file_path(text_document_uri) else {
        return false;
    };

    let mut matched_workspace = false;
    for workspace_config in workspace_configs {
        if !text_document_path.starts_with(&workspace_config.workspace_folder_path) {
            continue;
        }

        matched_workspace = true;
        if !workspace_config.is_ignored(&text_document_path) {
            return false;
        }
    }

    matched_workspace
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use tombi_config::FilesOptions;

    use super::*;

    macro_rules! test_workspace_ignore {
        ($name:ident, $respect_ignore_files:literal, $expected:literal) => {
            #[test]
            fn $name() {
                let tempdir = tempdir().unwrap();
                let root = tempdir.path();
                fs::create_dir(root.join(".git")).unwrap();
                fs::write(root.join(".gitignore"), "**/.terraform/*\n").unwrap();
                let path = root.join("module/.terraform/modules/child.toml");
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, "invalid TOML\n").unwrap();

                let mut config = Config::default();
                config.files = Some(FilesOptions {
                    respect_ignore_files: $respect_ignore_files.into(),
                    ..Default::default()
                });
                let workspace_config = WorkspaceConfig {
                    workspace_folder_path: root.to_path_buf(),
                    config,
                    config_path: Some(root.join("tombi.toml")),
                };
                let uri = tombi_uri::Uri::from_file_path(&path).unwrap();

                assert_eq!(
                    is_workspace_ignored(&uri, std::slice::from_ref(&workspace_config)),
                    $expected
                );
            }
        };
    }

    test_workspace_ignore!(workspace_ignore_respects_gitignore, true, true);
    test_workspace_ignore!(workspace_ignore_disabled, false, false);

    macro_rules! test_workspace_diagnostic_enabled {
        ($name:ident, diagnostic = $diagnostic:expr, workspace_diagnostic = $workspace_diagnostic:expr, $expected:literal) => {
            #[test]
            fn $name() {
                let mut config = Config::default();
                config.lsp = Some(tombi_config::LspOptions {
                    diagnostic: $diagnostic.map(|enabled: bool| tombi_config::LspDiagnostic {
                        enabled: Some(enabled.into()),
                    }),
                    workspace_diagnostic: $workspace_diagnostic.map(|enabled: bool| {
                        tombi_config::LspWorkspaceDiagnostic {
                            enabled: Some(enabled.into()),
                        }
                    }),
                    ..Default::default()
                });

                assert_eq!(is_workspace_diagnostic_enabled(&config), $expected);
            }
        };
    }

    test_workspace_diagnostic_enabled!(
        workspace_diagnostic_enabled_by_default,
        diagnostic = None,
        workspace_diagnostic = None,
        true
    );

    // `lsp.diagnostic.enabled = false` must also stop workspace diagnostics,
    // otherwise the whole workspace is crawled and linted only to discard the results.
    test_workspace_diagnostic_enabled!(
        workspace_diagnostic_follows_disabled_diagnostic,
        diagnostic = Some(false),
        workspace_diagnostic = None,
        false
    );

    test_workspace_diagnostic_enabled!(
        workspace_diagnostic_disabled_even_if_explicitly_enabled,
        diagnostic = Some(false),
        workspace_diagnostic = Some(true),
        false
    );

    test_workspace_diagnostic_enabled!(
        workspace_diagnostic_disabled_alone,
        diagnostic = Some(true),
        workspace_diagnostic = Some(false),
        false
    );
}
