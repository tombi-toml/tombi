use tombi_ast::AstNode;

use super::{AppendSemanticTokens, SemanticTokensBuilder};

impl AppendSemanticTokens for tombi_ast::KeyValue {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for comment in self.leading_comments() {
            comment.append_semantic_tokens(builder);
        }

        if let Some(key) = self.keys() {
            key.append_semantic_tokens(builder)
        }

        if let Some(value) = self.value() {
            value.append_semantic_tokens(builder)
        }
    }
}
