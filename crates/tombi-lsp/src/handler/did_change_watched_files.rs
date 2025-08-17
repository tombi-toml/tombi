use tower_lsp_server::ls_types::lsp::DidChangeWatchedFilesParams;

pub async fn handle_did_change_watched_files(params: DidChangeWatchedFilesParams) {
    tracing::info!("handle_did_change_watched_files");
    tracing::trace!(?params);
}
