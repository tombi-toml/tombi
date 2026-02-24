use tombi_ast::{AstNode, DanglingCommentGroupOr};

use super::{AppendSemanticTokens, SemanticTokensBuilder};

impl AppendSemanticTokens for tombi_ast::Array {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        if let Some(trailing_comment) = self.bracket_start_trailing_comment() {
            trailing_comment.append_semantic_tokens(builder);
        }

        for comment_group in self.dangling_comment_groups() {
            for comment in comment_group.comments() {
                comment.append_semantic_tokens(builder);
            }
        }

        for group in self.value_with_comma_groups() {
            match group {
                DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                    for comment in comment_group.comments() {
                        comment.append_semantic_tokens(builder);
                    }
                }
                DanglingCommentGroupOr::ItemGroup(value_group) => {
                    for (value, comma) in value_group.values_with_comma() {
                        value.append_semantic_tokens(builder);
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
