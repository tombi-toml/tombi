use tombi_ast::{AstNode, DanglingCommentGroupOr};

use super::{AppendSemanticTokens, SemanticTokensBuilder};

impl AppendSemanticTokens for tombi_ast::InlineTable {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        if let Some(trailing_comment) = self.brace_start_trailing_comment() {
            trailing_comment.append_semantic_tokens(builder);
        }

        for comment_group in self.dangling_comment_groups() {
            for comment in comment_group.comments() {
                comment.append_semantic_tokens(builder);
            }
        }

        for group in self.key_value_with_comma_groups() {
            match group {
                DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                    for comment in comment_group.comments() {
                        comment.append_semantic_tokens(builder);
                    }
                }
                DanglingCommentGroupOr::ItemGroup(key_value_group) => {
                    for (key_value, comma) in key_value_group.key_values_with_comma() {
                        key_value.append_semantic_tokens(builder);
                        if let Some(comma) = comma {
                            for comment in comma.leading_comments() {
                                comment.append_semantic_tokens(builder);
                            }

                            if let Some(comment) = comma.trailing_comment() {
                                comment.append_semantic_tokens(builder);
                            }
                        }
                    }
                }
            }
        }
    }
}
