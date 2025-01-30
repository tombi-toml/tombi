use tower_lsp::lsp_types::{CompletionParams, TextDocumentPositionParams};

use crate::{
    backend,
    completion::{get_completion_contents, CompletionContent},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_completion(
    backend: &backend::Backend,
    CompletionParams {
        text_document_position:
            TextDocumentPositionParams {
                text_document,
                position,
            },
        ..
    }: CompletionParams,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_completion");

    let config = backend.config().await;

    if !config
        .server
        .and_then(|server| server.completion)
        .and_then(|completion| completion.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.completion.enabled` is false");
        return Ok(None);
    }

    if !config
        .schema
        .and_then(|s| s.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`schema.enabled` is false");
        return Ok(None);
    }

    let Ok(Some(document_schema)) = &backend
        .schema_store
        .try_get_schema_from_url(&text_document.uri)
        .await
    else {
        tracing::debug!("schema not found: {}", text_document.uri);
        return Ok(None);
    };
    let Some(document_source) = backend.get_document_source(&text_document.uri) else {
        return Ok(None);
    };

    // FIXME: Remove whitespaces, because the AST assigns the whitespace to the next section.
    //        In the future, it would be better to move the whitespace in ast_editor.
    let mut position: text::Position = position.into();
    while position.column() != 0 && position.char_at_left(&document_source.source) == Some(' ') {
        position = text::Position::new(position.line(), position.column() - 1);
    }

    let toml_version = backend.toml_version().await.unwrap_or_default();
    let Some(root) = backend.get_incomplete_ast(&text_document.uri, toml_version) else {
        return Ok(None);
    };

    Ok(Some(get_completion_contents(
        root,
        position,
        document_schema,
        toml_version,
    )))
}
