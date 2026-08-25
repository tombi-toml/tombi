use tombi_ast_syntax::AstToken;

use crate::semantic_tokens::{AppendSemanticTokens, SemanticTokensBuilder, TokenType};

impl AppendSemanticTokens for tombi_ast_syntax::Comment {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        if let Some(tombi_value_directive) = self.get_tombi_value_directive() {
            builder.add_comment_directive(self, tombi_value_directive.directive_range);
        } else {
            builder.add_token(TokenType::COMMENT, self.syntax().range());
        }
    }
}
