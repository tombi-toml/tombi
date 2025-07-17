use tower_lsp::lsp_types::{SemanticTokens, SemanticTokensParams, SemanticTokensResult};

use crate::{
    backend::Backend,
    semantic_tokens::{AppendSemanticTokens, SemanticTokensBuilder},
};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_semantic_tokens_full(
    backend: &Backend,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_semantic_tokens_full");
    tracing::trace!(?params);

    let SemanticTokensParams { text_document, .. } = params;

    let Some((root, _)) = backend.get_ast_and_diagnostics(&text_document.uri).await else {
        return Ok(None);
    };

    let file_schema_range = root
        .file_schema_url(text_document.uri.to_file_path().ok().as_deref())
        .map(|(_, range)| range);

    let mut tokens_builder = SemanticTokensBuilder::new(file_schema_range);
    root.append_semantic_tokens(&mut tokens_builder);
    let tokens = tokens_builder.build();

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })))
}
