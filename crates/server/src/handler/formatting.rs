use config::FormatOptions;
use itertools::Either;
use tower_lsp::lsp_types::{
    notification::PublishDiagnostics, DocumentFormattingParams, PublishDiagnosticsParams, TextEdit,
};

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_formatting(
    backend: &Backend,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_formatting");
    tracing::trace!(?params);

    let DocumentFormattingParams { text_document, .. } = params;

    let config = backend.config().await;

    if !config
        .server
        .and_then(|server| server.formatting)
        .and_then(|formatting| formatting.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.formatting.enabled` is false");
        return Ok(None);
    }

    let (toml_version, _) = backend.text_document_toml_version(&text_document.uri).await;
    let mut document_sources = backend.document_sources.write().await;
    let Some(document_source) = document_sources.get_mut(&text_document.uri) else {
        return Ok(None);
    };

    match formatter::Formatter::new(
        toml_version,
        Default::default(),
        backend
            .config()
            .await
            .format
            .as_ref()
            .unwrap_or(&FormatOptions::default()),
        Some(Either::Left(&text_document.uri)),
        &backend.schema_store,
    )
    .format(&document_source.source)
    .await
    {
        Ok(new_text) => {
            if new_text != document_source.source {
                document_source.source = new_text.clone();

                return Ok(Some(vec![TextEdit {
                    range: text::Range::new(text::Position::MIN, text::Position::MAX).into(),
                    new_text,
                }]));
            } else {
                tracing::debug!("no change");
                backend
                    .client
                    .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                        uri: text_document.uri,
                        diagnostics: Vec::with_capacity(0),
                        version: Some(document_source.version),
                    })
                    .await;
            }
        }
        Err(diagnostics) => {
            tracing::error!("failed to format");
            backend
                .client
                .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: text_document.uri,
                    diagnostics: diagnostics.into_iter().map(Into::into).collect(),
                    version: Some(document_source.version),
                })
                .await;
        }
    }

    Ok(None)
}
