mod cache;

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
) -> Result<(), tower_lsp::jsonrpc::Error> {
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
