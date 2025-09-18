use itertools::Itertools;
use tombi_ast::AstNode;

use super::{AppendSemanticTokens, SemanticTokensBuilder, TokenType};

impl AppendSemanticTokens for tombi_ast::Table {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for comment in self.header_leading_comments() {
            comment.append_semantic_tokens(builder);
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

        if let Some(comment) = self.header_trailing_comment() {
            comment.append_semantic_tokens(builder);
        }

        let key_values = self.key_values().collect_vec();

        if key_values.is_empty() {
            for comments in self.key_values_dangling_comments() {
                for comment in comments {
                    comment.append_semantic_tokens(builder);
                }
            }
        } else {
            for comments in self.key_values_begin_dangling_comments() {
                for comment in comments {
                    comment.append_semantic_tokens(builder);
                }
            }

            for key_value in key_values {
                key_value.append_semantic_tokens(builder);
            }

            for comments in self.key_values_end_dangling_comments() {
                for comment in comments {
                    comment.append_semantic_tokens(builder);
                }
            }
        }
    }
}
