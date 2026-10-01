use tower_lsp::lsp_types::{SemanticTokens, SemanticTokensParams, SemanticTokensResult};

use crate::{
    backend::Backend,
    semantic_tokens::{AppendSemanticTokens, SemanticTokensBuilder},
};

pub async fn handle_semantic_tokens_full(
    backend: &Backend,
    params: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>, tower_lsp::jsonrpc::Error> {
    log::info!("handle_semantic_tokens_full");
    log::trace!("{:?}", params);

    let SemanticTokensParams { text_document, .. } = params;
    let text_document_uri: tombi_uri::Uri = text_document.uri.into();

    let Some(document_source) = backend
        .document_sources
        .read()
        .await
        .get(&text_document_uri)
        .cloned()
    else {
        return Ok(None);
    };
    let line_index = document_source.line_index();
    let encoding = document_source.encoding_kind();

    let mut tokens_builder = SemanticTokensBuilder::new(text_document_uri, line_index, encoding);

    document_source
        .ast()
        .append_semantic_tokens(&mut tokens_builder);

    let tokens = tokens_builder.build();

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })))
}
