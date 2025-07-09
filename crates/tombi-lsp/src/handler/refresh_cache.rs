use std::borrow::Cow;

use crate::Backend;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshCacheParams {}

pub async fn handle_refresh_cache(
    backend: &Backend,
    _params: RefreshCacheParams,
) -> Result<bool, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_refresh_cache");

    match backend
        .schema_store
        .refresh_cache(
            &backend.config().await,
            backend.config_path().await.as_deref(),
        )
        .await
    {
        Ok(true) => {
            tracing::info!("Cache refreshed");
            Ok(true)
        }
        Ok(false) => {
            tracing::info!("No cache to refresh");
            Ok(false)
        }
        Err(err) => {
            tracing::error!("Failed to refresh cache: {err}");
            Err(tower_lsp::jsonrpc::Error {
                code: tower_lsp::jsonrpc::ErrorCode::InternalError,
                message: Cow::Owned(format!("Failed to refresh cache: {err}")),
                data: None,
            })
        }
    }
}
