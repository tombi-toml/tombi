use tombi_schema_store::SchemaUri;
use tower_lsp_server::gen_lsp_types::{
    MessageType, ShowMessageNotification, ShowMessageParams, TextDocumentIdentifier,
};

use crate::{
    backend::Backend,
    workspace_diagnostic::{WorkspaceDiagnosticOptions, push_workspace_diagnostics},
};

pub async fn handle_update_schema(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<bool, tower_lsp_server::jsonrpc::Error> {
    log::info!("handle_update_schema");
    log::trace!("{:?}", params);

    let TextDocumentIdentifier { uri, .. } = params;

    match backend.config_manager.update_schema(SchemaUri::from(uri)).await {
        Ok(is_updated) => {
            if is_updated {
                backend.workspace_diagnostics_cache.write().await.clear_all();

                // Refresh workspace diagnostics after schema update
                // Include open files to ensure diagnostics are updated for all files, including those open in the editor
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
            }
            Ok(is_updated)
        }
        Err(err) => {
            backend
                .client
                .send_notification::<ShowMessageNotification>(ShowMessageParams {
                    kind: MessageType::Error,
                    message: err.to_string(),
                })
                .await;

            Ok(false)
        }
    }
}
