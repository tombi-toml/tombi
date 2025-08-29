use tombi_ast::AstNode;

use super::{AppendSemanticTokens, SemanticTokensBuilder};

impl AppendSemanticTokens for tombi_ast::InlineTable {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for comments in self.inner_begin_dangling_comments() {
            for comment in comments {
                comment.append_semantic_tokens(builder);
            }
        }

        for (key_value, comma) in self.key_values_with_comma() {
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

        for comments in self.inner_end_dangling_comments() {
            for comment in comments {
                comment.append_semantic_tokens(builder);
            }
        }
    }
}
