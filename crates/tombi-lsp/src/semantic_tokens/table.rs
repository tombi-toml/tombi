use tombi_ast::{AstNode, AstToken};

use super::{AppendSemanticTokens, SemanticTokensBuilder, TokenType};

impl AppendSemanticTokens for tombi_ast::Table {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for comment in self.header_leading_comments() {
            builder.add_token(TokenType::COMMENT, comment.as_ref().syntax().clone().into());
        }

        if let Some(token) = self.bracket_start() {
            builder.add_token(TokenType::OPERATOR, token.into())
        }

        if let Some(header) = self.header() {
            for key in header.keys() {
                builder.add_token(TokenType::TABLE, key.syntax().clone().into());
            }
        }

        if let Some(token) = self.bracket_end() {
            builder.add_token(TokenType::OPERATOR, token.into())
        }

        if let Some(comment) = self.header_tailing_comment() {
            builder.add_token(TokenType::COMMENT, comment.as_ref().syntax().clone().into())
        }

        for key_value in self.key_values() {
            key_value.append_semantic_tokens(builder);
        }
    }
}
