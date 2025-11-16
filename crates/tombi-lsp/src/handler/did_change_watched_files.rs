use tower_lsp::lsp_types::{DidChangeWatchedFilesParams, FileChangeType};

use crate::{backend::Backend, handler::workspace_diagnostic::upsert_document_source};

use super::diagnostic::push_diagnostics;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_did_change_watched_files(
    backend: &Backend,
    params: DidChangeWatchedFilesParams,
) {
    tracing::info!("handle_did_change_watched_files");
    tracing::trace!(?params);

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
                if upsert_document_source(backend, uri.clone()).await {
                    push_diagnostics(backend, uri).await;
                }
            }
            _ => {
                tracing::debug!("Ignored file change type {:?} for URI: {}", change.typ, uri);
            }
        }
    }
}
