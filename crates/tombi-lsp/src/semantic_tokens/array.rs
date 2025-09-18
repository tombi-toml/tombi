use itertools::Itertools;
use tombi_ast::AstNode;

use super::{AppendSemanticTokens, SemanticTokensBuilder};

impl AppendSemanticTokens for tombi_ast::Array {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        let values_with_comma = self.values_with_comma().collect_vec();

        if values_with_comma.is_empty() {
            for comments in self.inner_dangling_comments() {
                for comment in comments {
                    comment.append_semantic_tokens(builder);
                }
            }
        } else {
            for comments in self.inner_begin_dangling_comments() {
                for comment in comments {
                    comment.append_semantic_tokens(builder);
                }
            }

            for (value, comma) in self.values_with_comma() {
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

            for comments in self.inner_end_dangling_comments() {
                for comment in comments {
                    comment.append_semantic_tokens(builder);
                }
            }
        }
    }
}
