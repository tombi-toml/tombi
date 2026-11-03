mod cache;

use std::path::PathBuf;

use tower_lsp_server::gen_lsp_types::{
    BaseUri, DidChangeWatchedFilesRegistrationOptions, FileSystemWatcher, GlobPattern, Pattern,
    Registration, RelativePattern, Unregistration, WatchKind,
};

use tombi_glob::search_pattern_matched_paths;

use crate::{
    Backend,
    diagnostic::{DiagnosticsResult, get_diagnostics_result},
    document::{DocumentSource, ParsedText},
    workspace_config::get_workspace_configs,
};
pub use cache::WorkspaceDiagnosticsCache;

#[derive(Debug, Default)]
pub struct WorkspaceDiagnosticOptions {
    pub include_open_files: bool,
}

pub async fn push_workspace_diagnostics(
    backend: &Backend,
    options: &WorkspaceDiagnosticOptions,
) -> Result<(), tower_lsp_server::jsonrpc::Error> {
    let targets = collect_workspace_diagnostic_targets(backend).await;

    if targets.is_empty() {
        return Ok(());
    }

    log::info!("push_workspace_diagnostics");
    log::trace!("{:?}", options);

    for text_document_uri in targets {
        publish_workspace_diagnostics(backend, text_document_uri, options).await;
    }

    Ok(())
}

/// Drop cached diagnostics and re-diagnose open documents and workspace targets
/// after the effective config or schemas changed.
pub async fn refresh_diagnostics_after_config_change(backend: &Backend) {
    backend.workspace_diagnostics_cache.write().await.reset();

    if backend.is_diagnostic_mode_push().await {
        // Open documents may be outside the workspace targets, so re-diagnose them explicitly.
        let open_document_uris = backend
            .document_sources
            .read()
            .await
            .iter()
            .filter_map(|(uri, source)| source.version.is_some().then_some(uri.clone()))
            .collect::<Vec<_>>();
        for uri in open_document_uris {
            backend.push_diagnostics(uri).await;
        }

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
    } else {
        backend.refresh_pull_diagnostics().await;
    }
}

/// Re-diagnose Cargo manifests when a file referenced by Cargo.toml changes.
pub async fn refresh_cargo_diagnostics_after_file_changes(
    backend: &Backend,
    changed_paths: &[PathBuf],
) -> bool {
    let cargo_documents = backend
        .document_sources
        .read()
        .await
        .iter()
        .filter_map(|(uri, source)| {
            let manifest_path = uri.to_file_path().ok()?;
            if manifest_path
                .file_name()
                .is_none_or(|name| name != "Cargo.toml")
            {
                return None;
            }

            let document_tree = source.document_tree();
            changed_paths
                .iter()
                .any(|path| {
                    tombi_extension_cargo::references_path(document_tree, &manifest_path, path)
                })
                .then(|| (uri.clone(), source.version.is_some()))
        })
        .collect::<Vec<_>>();

    if cargo_documents.is_empty() {
        return false;
    }

    {
        let mut cache = backend.workspace_diagnostics_cache.write().await;
        for (uri, _) in &cargo_documents {
            cache.clear(uri);
        }
    }

    if backend.is_diagnostic_mode_push().await {
        for (uri, is_open) in cargo_documents {
            if is_open {
                backend.push_diagnostics(uri).await;
            } else {
                publish_workspace_diagnostics(
                    backend,
                    uri,
                    &WorkspaceDiagnosticOptions {
                        include_open_files: false,
                    },
                )
                .await;
            }
        }
    }

    true
}

/// Keep client-side file watching limited to paths referenced by known Cargo manifests.
pub async fn refresh_cargo_file_watchers(backend: &Backend) {
    let paths = {
        let sources = backend.document_sources.read().await;
        sources
            .iter()
            .filter_map(|(uri, source)| {
                let manifest_path = uri.to_file_path().ok()?;
                if manifest_path
                    .file_name()
                    .is_none_or(|name| name != "Cargo.toml")
                {
                    return None;
                }
                Some(tombi_extension_cargo::referenced_paths(
                    source.document_tree(),
                    &manifest_path,
                ))
            })
            .flatten()
            .collect::<std::collections::HashSet<_>>()
    };

    let mut previous = backend.cargo_file_watchers.lock().await;
    if previous
        .iter()
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        == paths
    {
        return;
    }

    if !previous.is_empty()
        && let Err(error) = backend
            .client
            .unregister_capability(vec![Unregistration::new(
                "cargo-reference-file-watcher".to_string(),
                "workspace/didChangeWatchedFiles".to_string(),
            )])
            .await
    {
        log::warn!("failed to unregister Cargo reference file watchers: {error}");
        return;
    }

    if !paths.is_empty() {
        let watchers = paths
            .iter()
            .filter_map(|path| {
                let (base, relative_path) = watcher_base_and_pattern(path)?;
                let base_uri = tombi_uri::Uri::from_file_path(base).ok()?;
                Some(FileSystemWatcher {
                    glob_pattern: GlobPattern::RelativePattern(RelativePattern::new(
                        BaseUri::Uri(base_uri.into()),
                        Pattern::from(relative_path),
                    )),
                    kind: Some(WatchKind::Create | WatchKind::Change | WatchKind::Delete),
                })
            })
            .collect();
        let registration = Registration {
            id: "cargo-reference-file-watcher".to_string(),
            method: "workspace/didChangeWatchedFiles".to_string(),
            register_options: Some(serde_json::json!(
                DidChangeWatchedFilesRegistrationOptions { watchers }
            )),
        };
        if let Err(error) = backend.client.register_capability(vec![registration]).await {
            log::warn!("failed to register Cargo reference file watchers: {error}");
            return;
        }
    }

    *previous = paths.into_iter().collect();
}

fn watcher_base_and_pattern(path: &std::path::Path) -> Option<(&std::path::Path, String)> {
    let mut base = path.parent()?;
    while !base.is_dir() {
        base = base.parent()?;
    }
    let relative_path = path.strip_prefix(base).ok()?;
    let pattern = relative_path
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()?
        .join("/");
    Some((base, pattern))
}

pub async fn collect_workspace_diagnostic_targets(backend: &Backend) -> Vec<tombi_uri::Uri> {
    if let Some(targets) = backend
        .workspace_diagnostics_cache
        .read()
        .await
        .workspace_targets()
    {
        return targets;
    }

    let Some(configs) = get_workspace_configs(backend).await else {
        return Vec::new();
    };

    let mut candidates = tombi_hashmap::HashSet::new();
    let home_dir = tombi_fs::home_dir();

    for workspace_config in configs {
        if !workspace_config.is_workspace_diagnostic_enabled() {
            log::debug!(
                "`lsp.workspace-diagnostic.enabled` is false in {}",
                workspace_config.workspace_folder_path.display()
            );
            continue;
        }

        if let Some(home_dir) = &home_dir
            && &workspace_config.workspace_folder_path == home_dir
        {
            log::debug!(
                "skip diagnostics for workspace folder matching $HOME: {:?}",
                workspace_config.workspace_folder_path
            );
            continue;
        }

        let files_options = workspace_config.config.files.clone().unwrap_or_default();

        for matched_path in
            search_pattern_matched_paths(workspace_config.workspace_folder_path, files_options)
                .await
        {
            let tombi_glob::FileSearchEntry::Found(path) = matched_path else {
                continue;
            };

            if let Ok(uri) = tombi_uri::Uri::from_file_path(path) {
                candidates.insert(uri);
            }
        }
    }

    let mut targets = Vec::with_capacity(candidates.len());

    for target in candidates {
        if upsert_document_source(backend, target.clone()).await {
            targets.push(target);
        }
    }

    backend
        .workspace_diagnostics_cache
        .write()
        .await
        .set_workspace_targets(targets.clone().into_iter().collect());

    targets
}

async fn publish_workspace_diagnostics(
    backend: &Backend,
    text_document_uri: tombi_uri::Uri,
    options: &WorkspaceDiagnosticOptions,
) {
    let Some(diagnostics_result) = get_diagnostics_result(backend, &text_document_uri).await else {
        return;
    };

    log::trace!("{:?}", diagnostics_result);

    let DiagnosticsResult {
        diagnostics,
        version,
    } = diagnostics_result;

    if !options.include_open_files && version.is_some() {
        log::debug!(
            "skip publishing workspace diagnostics because version is some: {text_document_uri}"
        );
        return;
    }

    backend
        .client
        .publish_diagnostics(text_document_uri.into(), diagnostics, version)
        .await
}

pub async fn upsert_document_source(backend: &Backend, text_document_uri: tombi_uri::Uri) -> bool {
    let text_document_path = match text_document_uri.to_file_path() {
        Ok(text_document_path) => text_document_path,
        Err(_) => {
            log::warn!("watcher event for non-file URI: {text_document_uri}");
            return false;
        }
    };

    let Ok(content) = tombi_fs::read_to_string_async(&text_document_path).await else {
        log::warn!(
            "failed to read file for diagnostics: {:?}",
            text_document_path
        );
        return false;
    };

    let parsed = ParsedText::parse(content);
    let toml_version = backend
        .text_document_toml_version(&text_document_uri, &parsed.root())
        .await;
    let encoding_kind = backend.capabilities.read().await.encoding_kind;

    {
        let mut document_sources = backend.document_sources.write().await;
        if let Some(source) = document_sources.get_mut(&text_document_uri) {
            if source.version.is_some() {
                log::debug!("skip diagnostics for open document: {text_document_uri}");
                return true;
            }

            *source = DocumentSource::new(parsed, None, toml_version, encoding_kind);
        } else {
            document_sources.insert(
                text_document_uri.clone(),
                DocumentSource::new(parsed, None, toml_version, encoding_kind),
            );
        }
    }

    true
}
