use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType};

use crate::{backend::Backend, document::DocumentSource};

use super::diagnostic::push_diagnostics;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_did_change_watched_files(
    backend: &Backend,
    params: DidChangeWatchedFilesParams,
) {
    tracing::info!("handle_did_change_watched_files");
    tracing::trace!(?params);

    let encoding_kind = backend.capabilities.read().await.encoding_kind;

    for change in params.changes {
        let uri: tombi_uri::Uri = change.uri.clone().into();

        tracing::debug!("Detected {:?} via watcher: {}", change.typ, uri);

        match change.typ {
            FileChangeType::DELETED => {
                {
                    let mut document_sources = backend.document_sources.write().await;
                    document_sources.remove(&uri);
                }

                backend
                    .client
                    .publish_diagnostics(change.uri, Vec::with_capacity(0), None)
                    .await;
            }
            FileChangeType::CREATED | FileChangeType::CHANGED => {
                let file_path = match uri.to_file_path() {
                    Ok(file_path) => file_path,
                    Err(_) => {
                        tracing::warn!("Watcher event for non-file URI: {}", uri);
                        continue;
                    }
                };

                let Ok(content) = tokio::fs::read_to_string(&file_path).await else {
                    tracing::warn!("Failed to read file for diagnostics: {:?}", file_path);
                    continue;
                };

                let toml_version = backend.text_document_toml_version(&uri, &content).await;

                let mut document_sources = backend.document_sources.write().await;
                if let Some(source) = document_sources.get_mut(&uri) {
                    if source.version.is_some() {
                        tracing::debug!("Skip watcher diagnostics for open document: {}", uri);
                        continue;
                    }

                    source.set_text(content, toml_version);
                } else {
                    document_sources.insert(
                        uri.clone(),
                        DocumentSource::new(content, None, toml_version, encoding_kind),
                    );
                }
                drop(document_sources);

                push_diagnostics(backend, uri.clone(), None).await;
            }
            _ => {
                tracing::debug!("Ignored file change type {:?} for URI: {}", change.typ, uri);
            }
        }
    }
}
