use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType};

use crate::{
    backend::Backend,
    workspace_config::{WorkspaceConfig, get_workspace_configs, is_workspace_target},
    workspace_diagnostic::upsert_document_source,
};

use super::diagnostic::push_diagnostics;

pub async fn handle_did_change_watched_files(
    backend: &Backend,
    params: DidChangeWatchedFilesParams,
) {
    log::info!("handle_did_change_watched_files");
    log::trace!("{:?}", params);

    let mut should_refresh_pull_diagnostics = false;
    let home_dir = dirs::home_dir();
    let mut workspace_configs: Option<Vec<WorkspaceConfig>> = None;

    for change in params.changes {
        let uri: tombi_uri::Uri = change.uri.clone().into();

        log::debug!("Detected {:?} via watcher: {}", change.typ, uri);

        match change.typ {
            FileChangeType::DELETED => {
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
                log::debug!("Ignored file change type {:?} for URI: {}", change.typ, uri);
            }
        }
    }

    if should_refresh_pull_diagnostics {
        backend.refresh_pull_diagnostics().await;
    }
}
