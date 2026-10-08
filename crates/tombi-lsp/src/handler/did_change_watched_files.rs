use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType, TextDocumentIdentifier};

use crate::{
    backend::Backend,
    workspace_config::{
        WorkspaceConfig, get_workspace_configs, is_workspace_ignored, is_workspace_target,
    },
    workspace_diagnostic::{
        WorkspaceDiagnosticOptions, push_workspace_diagnostics, upsert_document_source,
    },
};

use super::diagnostic::push_diagnostics;

pub async fn handle_did_change_watched_files(
    backend: &Backend,
    params: DidChangeWatchedFilesParams,
) {
    log::info!("handle_did_change_watched_files");
    log::trace!("{:?}", params);

    let mut should_refresh_pull_diagnostics = false;
    let mut config_reloaded = false;
    let home_dir = tombi_fs::home_dir();
    let mut workspace_configs: Option<Vec<WorkspaceConfig>> = None;

    for change in params.changes {
        let uri: tombi_uri::Uri = change.uri.clone().into();

        log::debug!("detected {:?} via watcher: {}", change.typ, uri);

        if matches!(
            change.typ,
            FileChangeType::CREATED | FileChangeType::CHANGED
        ) && is_config_file(&uri)
        {
            // Config files changed outside the editor are reloaded as on save.
            let identifier = TextDocumentIdentifier::new(change.uri.clone());
            if matches!(
                crate::handler::handle_update_config(backend, identifier).await,
                Ok(true)
            ) {
                config_reloaded = true;
            }
        }

        if matches!(
            change.typ,
            FileChangeType::CREATED | FileChangeType::CHANGED
        ) {
            if workspace_configs.is_none() {
                workspace_configs = Some(get_workspace_configs(backend).await.unwrap_or_default());
            }

            if is_workspace_ignored(&uri, workspace_configs.as_deref().unwrap_or(&[])) {
                backend.wait_for_document_open(&uri).await;
                let mut document_sources = backend.document_sources.write().await;
                if document_sources
                    .get(&uri)
                    .is_none_or(|source| source.version.is_none())
                {
                    log::debug!("clear watcher diagnostics for ignored file: {uri}");
                    document_sources.remove(&uri);
                    backend
                        .workspace_diagnostics_cache
                        .write()
                        .await
                        .untrack(&uri);

                    if backend.is_diagnostic_mode_push().await {
                        backend
                            .client
                            .publish_diagnostics(change.uri, Vec::new(), None)
                            .await;
                    } else {
                        should_refresh_pull_diagnostics = true;
                    }
                    continue;
                }
            }
        }

        match change.typ {
            FileChangeType::DELETED => {
                if let Ok(path) = uri.to_file_path() {
                    tombi_fs::remove_file(&path);
                }
                {
                    let mut document_sources = backend.document_sources.write().await;
                    document_sources.remove(&uri);
                }

                if backend.is_diagnostic_mode_push().await {
                    backend
                        .client
                        .publish_diagnostics(change.uri, Vec::new(), None)
                        .await;
                } else {
                    should_refresh_pull_diagnostics = true;
                }

                backend
                    .workspace_diagnostics_cache
                    .write()
                    .await
                    .untrack(&uri);
            }
            FileChangeType::CREATED => {
                if upsert_document_source(backend, uri.clone()).await {
                    if workspace_configs.is_none() {
                        workspace_configs =
                            Some(get_workspace_configs(backend).await.unwrap_or_default());
                    }

                    let is_workspace_target = is_workspace_target(
                        &uri,
                        workspace_configs.as_deref().unwrap_or(&[]),
                        home_dir.as_deref(),
                    );

                    backend
                        .workspace_diagnostics_cache
                        .write()
                        .await
                        .track(uri.clone(), is_workspace_target);

                    push_diagnostics(backend, uri).await;
                    should_refresh_pull_diagnostics = true;
                }
            }
            FileChangeType::CHANGED => {
                if upsert_document_source(backend, uri.clone()).await {
                    backend
                        .workspace_diagnostics_cache
                        .write()
                        .await
                        .clear(&uri);

                    push_diagnostics(backend, uri).await;
                    should_refresh_pull_diagnostics = true;
                }
            }
            _ => {
                log::debug!("ignored file change type {:?} for URI: {}", change.typ, uri);
            }
        }
    }

    if config_reloaded {
        if let Err(err) = push_workspace_diagnostics(
            backend,
            &WorkspaceDiagnosticOptions {
                include_open_files: true,
            },
        )
        .await
        {
            log::warn!("failed to push workspace diagnostics: {err}");
        }
        should_refresh_pull_diagnostics = true;
    }

    if should_refresh_pull_diagnostics {
        backend.refresh_pull_diagnostics().await;
    }
}

fn is_config_file(uri: &tombi_uri::Uri) -> bool {
    uri.to_file_path()
        .ok()
        .and_then(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| tombi_config::SUPPORTED_CONFIG_FILENAMES.contains(&name))
        })
        .unwrap_or(false)
}
