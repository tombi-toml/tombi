use tombi_ast::AstNode;

use super::{AppendSemanticTokens, SemanticTokensBuilder, TokenType};

impl AppendSemanticTokens for tombi_ast::Keys {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for key in self.keys() {
            key.append_semantic_tokens(builder);
        }
    }
}

impl AppendSemanticTokens for tombi_ast::Key {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        builder.add_token(TokenType::KEY, self.syntax().clone().into());
    }
}
