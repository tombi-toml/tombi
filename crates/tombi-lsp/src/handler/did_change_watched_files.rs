use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType, TextDocumentIdentifier};

use crate::{
    backend::Backend,
    workspace_config::{
        WorkspaceConfig, get_workspace_configs, is_workspace_ignored, is_workspace_target,
    },
    workspace_diagnostic::{refresh_diagnostics_after_config_change, upsert_document_source},
};

use super::diagnostic::push_diagnostics;

pub async fn handle_did_change_watched_files(
    backend: &Backend,
    params: DidChangeWatchedFilesParams,
) {
    log::info!("handle_did_change_watched_files");
    log::trace!("{:?}", params);

    let mut should_refresh_pull_diagnostics = false;
    let mut config_changed = false;
    let mut schema_changed = false;
    let home_dir = tombi_fs::home_dir();
    let mut workspace_configs: Option<Vec<WorkspaceConfig>> = None;

    for change in params.changes {
        let uri: tombi_uri::Uri = change.uri.clone().into();

        log::debug!("detected {:?} via watcher: {}", change.typ, uri);

        if is_json_file(&uri) {
            // Only schemas already loaded by the schema stores are updated.
            if matches!(
                change.typ,
                FileChangeType::CREATED | FileChangeType::CHANGED
            ) {
                match backend
                    .config_manager
                    .update_schema(tombi_schema_store::SchemaUri::from(change.uri))
                    .await
                {
                    Ok(updated) => schema_changed |= updated,
                    Err(err) => log::warn!("failed to update schema {uri}: {err}"),
                }
            }
            continue;
        }

        if is_config_file(&uri) {
            // Config files changed outside the editor are reloaded as on save.
            // Config discovery must be redone since a config may have been added or removed.
            backend.config_manager.clear_source_config_paths().await;
            config_changed = true;
            if matches!(
                change.typ,
                FileChangeType::CREATED | FileChangeType::CHANGED
            ) {
                let identifier = TextDocumentIdentifier::new(change.uri.clone());
                if let Err(err) = crate::handler::handle_update_config(backend, identifier).await {
                    log::warn!("failed to reload config: {err}");
                }
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

    if config_changed || schema_changed {
        refresh_diagnostics_after_config_change(backend).await;
    }

    if should_refresh_pull_diagnostics {
        backend.refresh_pull_diagnostics().await;
    }
}

fn is_config_file(uri: &tombi_uri::Uri) -> bool {
    uri.to_file_path().ok().is_some_and(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| tombi_config::SUPPORTED_CONFIG_FILENAMES.contains(&name))
    })
}

fn is_json_file(uri: &tombi_uri::Uri) -> bool {
    uri.to_file_path()
        .ok()
        .is_some_and(|path| path.extension().is_some_and(|ext| ext == "json"))
}
