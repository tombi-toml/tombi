use tombi_ast::{AstNode, DanglingCommentGroupOr};

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

        for comment_group in self.dangling_comment_groups() {
            for comment in comment_group.comments() {
                comment.append_semantic_tokens(builder);
            }
        }

        for group in self.key_value_groups() {
            match group {
                DanglingCommentGroupOr::ItemGroup(key_value_group) => {
                    for key_value in key_value_group.key_values() {
                        key_value.append_semantic_tokens(builder);
                    }
                }
                DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                    for comment in comment_group.comments() {
                        comment.append_semantic_tokens(builder);
                    }
                }
            }
        }
    }
}
