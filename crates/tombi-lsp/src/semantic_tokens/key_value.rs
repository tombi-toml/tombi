use tombi_ast::AstNode;

use crate::semantic_tokens::TokenType;

use super::{AppendSemanticTokens, SemanticTokensBuilder};

impl AppendSemanticTokens for tombi_ast::KeyValue {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for comment in self.leading_comments() {
            comment.append_semantic_tokens(builder);
        }

        if let Some(key) = self.keys() {
            key.append_semantic_tokens(builder)
        }

        if let Some(token) = self.eq() {
            builder.add_token(TokenType::OPERATOR, token.into())
        }

        if let Some(value) = self.value() {
            value.append_semantic_tokens(builder)
        }
    }
}
