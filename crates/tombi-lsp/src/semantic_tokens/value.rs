use tombi_ast_syntax::AstNode;

use super::{AppendSemanticTokens, SemanticTokensBuilder, TokenType};

impl AppendSemanticTokens for tombi_ast_syntax::Value<'_> {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for comment in self.leading_comments() {
            comment.append_semantic_tokens(builder);
        }

        match self {
            Self::BasicString(n) => builder.add_token(TokenType::STRING, n.token().unwrap().span()),
            Self::LiteralString(n) => {
                builder.add_token(TokenType::STRING, n.token().unwrap().span())
            }
            Self::MultiLineBasicString(n) => {
                builder.add_token(TokenType::STRING, n.token().unwrap().span())
            }
            Self::MultiLineLiteralString(n) => {
                builder.add_token(TokenType::STRING, n.token().unwrap().span())
            }
            Self::IntegerBin(n) => builder.add_token(TokenType::NUMBER, n.token().unwrap().span()),
            Self::IntegerOct(n) => builder.add_token(TokenType::NUMBER, n.token().unwrap().span()),
            Self::IntegerDec(n) => builder.add_token(TokenType::NUMBER, n.token().unwrap().span()),
            Self::IntegerHex(n) => builder.add_token(TokenType::NUMBER, n.token().unwrap().span()),
            Self::Float(n) => builder.add_token(TokenType::NUMBER, n.token().unwrap().span()),
            Self::Boolean(n) => builder.add_token(TokenType::BOOLEAN, n.token().unwrap().span()),
            Self::OffsetDateTime(n) => {
                builder.add_token(TokenType::OFFSET_DATE_TIME, n.token().unwrap().span())
            }
            Self::LocalDateTime(n) => {
                builder.add_token(TokenType::LOCAL_DATE_TIME, n.token().unwrap().span())
            }
            Self::LocalDate(n) => {
                builder.add_token(TokenType::LOCAL_DATE, n.token().unwrap().span())
            }
            Self::LocalTime(n) => {
                builder.add_token(TokenType::LOCAL_TIME, n.token().unwrap().span())
            }
            Self::Array(array) => array.append_semantic_tokens(builder),
            Self::InlineTable(inline_table) => inline_table.append_semantic_tokens(builder),
        }

        if let Some(comment) = self.trailing_comment() {
            comment.append_semantic_tokens(builder);
        }
    }
}
