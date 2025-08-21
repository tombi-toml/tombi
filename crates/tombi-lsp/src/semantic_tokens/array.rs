use tombi_ast::{AstNode, AstToken};

use super::{AppendSemanticTokens, SemanticTokensBuilder, TokenType};

impl AppendSemanticTokens for tombi_ast::Array {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for comments in self.inner_begin_dangling_comments() {
            for comment in comments {
                builder.add_token(TokenType::COMMENT, comment.as_ref().syntax().clone().into());
            }
        }

        for (value, comma) in self.values_with_comma() {
            value.append_semantic_tokens(builder);
            if let Some(comma) = comma {
                for comment in comma.leading_comments() {
                    builder.add_token(TokenType::COMMENT, comment.as_ref().syntax().clone().into());
                }

                if let Some(comment) = comma.trailing_comment() {
                    builder.add_token(TokenType::COMMENT, comment.as_ref().syntax().clone().into())
                }
            }
        }

        for comments in self.inner_end_dangling_comments() {
            for comment in comments {
                builder.add_token(TokenType::COMMENT, comment.as_ref().syntax().clone().into());
            }
        }
    }
}
