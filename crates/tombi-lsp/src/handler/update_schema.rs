use tombi_schema_store::SchemaUri;
use tower_lsp::lsp_types::{
    MessageType, ShowMessageParams, TextDocumentIdentifier, notification::ShowMessage,
};

use crate::{
    backend::Backend,
    handler::workspace_diagnostic::{WorkspaceDiagnosticOptions, push_workspace_diagnostics},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_update_schema(
    backend: &Backend,
    params: TextDocumentIdentifier,
) -> Result<bool, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_update_schema");
    tracing::trace!(?params);

    let TextDocumentIdentifier { uri, .. } = params;

    match backend
        .config_manager
        .update_schema(&SchemaUri::from(uri))
        .await
    {
        Ok(is_updated) => {
            if is_updated {
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
                    tracing::warn!("Failed to push workspace diagnostics: {err}");
                }
            }
            Ok(is_updated)
        }
        Err(err) => {
            backend
                .client
                .send_notification::<ShowMessage>(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: err.to_string(),
                })
                .await;

            Ok(false)
        }
    }
}
