use tower_lsp::lsp_types::{
    notification::ShowMessage, MessageType, ShowMessageParams, TextDocumentIdentifier,
};

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_update_schema(
    backend: &Backend,
    TextDocumentIdentifier {
        uri: schema_url, ..
    }: TextDocumentIdentifier,
) -> Result<bool, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_update_schema");

    match backend.schema_store.update_schema(&schema_url).await {
        Ok(is_updated) => Ok(is_updated),
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
