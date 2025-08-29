use tombi_ast::AstNode;

use super::{AppendSemanticTokens, SemanticTokensBuilder, TokenType};

impl AppendSemanticTokens for tombi_ast::Value {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for comment in self.leading_comments() {
            comment.append_semantic_tokens(builder);
        }

        match self {
            Self::BasicString(n) => {
                builder.add_token(TokenType::STRING, (n.token().unwrap()).into())
            }
            Self::LiteralString(n) => {
                builder.add_token(TokenType::STRING, (n.token().unwrap()).into())
            }
            Self::MultiLineBasicString(n) => {
                builder.add_token(TokenType::STRING, (n.token().unwrap()).into())
            }
            Self::MultiLineLiteralString(n) => {
                builder.add_token(TokenType::STRING, (n.token().unwrap()).into())
            }
            Self::IntegerBin(n) => {
                builder.add_token(TokenType::NUMBER, (n.token().unwrap()).into())
            }
            Self::IntegerOct(n) => {
                builder.add_token(TokenType::NUMBER, (n.token().unwrap()).into())
            }
            Self::IntegerDec(n) => {
                builder.add_token(TokenType::NUMBER, (n.token().unwrap()).into())
            }
            Self::IntegerHex(n) => {
                builder.add_token(TokenType::NUMBER, (n.token().unwrap()).into())
            }
            Self::Float(n) => builder.add_token(TokenType::NUMBER, (n.token().unwrap()).into()),
            Self::Boolean(n) => builder.add_token(TokenType::BOOLEAN, (n.token().unwrap()).into()),
            Self::OffsetDateTime(n) => {
                builder.add_token(TokenType::OFFSET_DATE_TIME, (n.token().unwrap()).into())
            }
            Self::LocalDateTime(n) => {
                builder.add_token(TokenType::LOCAL_DATE_TIME, (n.token().unwrap()).into())
            }
            Self::LocalDate(n) => {
                builder.add_token(TokenType::LOCAL_DATE, (n.token().unwrap()).into())
            }
            Self::LocalTime(n) => {
                builder.add_token(TokenType::LOCAL_TIME, (n.token().unwrap()).into())
            }
            Self::Array(array) => array.append_semantic_tokens(builder),
            Self::InlineTable(inline_table) => inline_table.append_semantic_tokens(builder),
        }

        if let Some(comment) = self.trailing_comment() {
            comment.append_semantic_tokens(builder);
        }
    }
}
