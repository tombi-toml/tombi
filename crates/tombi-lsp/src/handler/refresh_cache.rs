use std::borrow::Cow;

use crate::{Backend, workspace_diagnostic::refresh_diagnostics_after_config_change};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshCacheParams {}

pub async fn handle_refresh_cache(
    backend: &Backend,
    _params: RefreshCacheParams,
) -> Result<bool, tower_lsp::jsonrpc::Error> {
    log::info!("handle_refresh_cache");

    match backend.config_manager.refresh_cache().await {
        Ok(true) => {
            log::info!("cache refreshed");
            refresh_diagnostics_after_config_change(backend).await;
            Ok(true)
        }
        Ok(false) => {
            log::info!("no cache to refresh");
            Ok(false)
        }
        Err(err) => {
            log::error!("failed to refresh cache: {err}");
            Err(tower_lsp::jsonrpc::Error {
                code: tower_lsp::jsonrpc::ErrorCode::InternalError,

                message: Cow::Owned(format!("Failed to refresh cache: {err}")),
                data: None,
            })
        }
    }
}
