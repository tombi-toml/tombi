use ahash::AHashMap;
use itertools::{Either, Itertools};
use tombi_config::LintOptions;
use tombi_file_search::FileSearch;
use tombi_uri::{url_from_file_path, url_to_file_path};
use tower_lsp::lsp_types::{TextDocumentIdentifier, Url};

use crate::{backend::Backend, config_manager::ConfigSchemaStore};

pub async fn publish_diagnostics(backend: &Backend, text_document_uri: Url, version: Option<i32>) {
    #[derive(Debug)]
    struct PublishDiagnosticsParams {
        text_document: TextDocumentIdentifier,
        version: Option<i32>,
    }

    let params = PublishDiagnosticsParams {
        text_document: TextDocumentIdentifier {
            uri: text_document_uri,
        },
        version,
    };

    tracing::info!("publish_diagnostics");
    tracing::trace!(?params);

    let Some(diagnostics) = diagnostics(backend, &params.text_document.uri).await else {
        return;
    };

    backend
        .client
        .publish_diagnostics(params.text_document.uri, diagnostics, params.version)
        .await
}

pub async fn publish_workspace_diagnostics(backend: &Backend) {
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
                    .filter_map(|workspace| url_to_file_path(&workspace.uri).ok())
                    .collect_vec()
            });

    let Some(workspace_folder_paths) = workspace_folder_paths else {
        return;
    };

    let mut configs = AHashMap::new();
    for workspace_folder_path in workspace_folder_paths {
        let Some(workspace_folder_path_str) = workspace_folder_path.to_str() else {
            continue;
        };
        if let Ok((config, config_path, config_level)) =
            serde_tombi::config::load_with_path_and_level(Some(workspace_folder_path.clone()))
        {
            configs.entry(config_path).or_insert((config, config_level));
        };
        for (config_path, (config, config_level)) in &configs {
            if let FileSearch::Files(files) = tombi_file_search::FileSearch::new(
                &[workspace_folder_path_str],
                config,
                config_path.as_deref(),
                *config_level,
            )
            .await
            {
                tracing::debug!(
                    "Found {} files, in {}",
                    files.len(),
                    workspace_folder_path_str
                );
                tracing::debug!("Founded files: {:?}", files);

                for file in files {
                    if let Some(file_path) = file
                        .ok()
                        .and_then(|file_path| url_from_file_path(&file_path).ok())
                    {
                        let version = {
                            backend
                                .document_sources
                                .read()
                                .await
                                .get(&file_path)
                                .map(|document_source| document_source.version)
                        };

                        publish_diagnostics(backend, file_path, version).await;
                    }
                }
            }
        }
    }
}

async fn diagnostics(
    backend: &Backend,
    text_document_uri: &Url,
) -> Option<Vec<tower_lsp::lsp_types::Diagnostic>> {
    let ConfigSchemaStore {
        config,
        schema_store,
    } = backend
        .config_manager
        .config_schema_store_for_url(text_document_uri)
        .await;

    if !config
        .lsp()
        .and_then(|server| server.diagnostics.as_ref())
        .and_then(|diagnostics| diagnostics.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.diagnostics.enabled` is false");
        return None;
    }

    let root = backend.get_incomplete_ast(text_document_uri).await?;

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(text_document_uri)))
        .await
        .ok()
        .flatten();

    let document_tombi_comment_directive =
        tombi_comment_directive::get_document_tombi_comment_directive(&root).await;
    let (toml_version, _) = backend
        .source_toml_version(
            document_tombi_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let document_sources = backend.document_sources.read().await;

    match document_sources.get(text_document_uri) {
        Some(document) => tombi_linter::Linter::new(
            toml_version,
            config.lint.as_ref().unwrap_or(&LintOptions::default()),
            Some(Either::Left(text_document_uri)),
            &schema_store,
        )
        .lint(&document.text)
        .await
        .map_or_else(
            |diagnostics| Some(diagnostics.into_iter().unique().map(Into::into).collect()),
            |_| Some(Vec::with_capacity(0)),
        ),
        None => None,
    }
}
