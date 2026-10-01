use tombi_json_syntax::SyntaxKind::*;

#[derive(Debug, Default)]
pub struct Lexed {
    pub tokens: Vec<crate::Token>,
    pub errors: Vec<crate::Error>,
    /// The start offset of each line, to build a [`tombi_text::LineIndex`] without scanning again.
    pub line_starts: Vec<tombi_text::Offset>,
}

impl Lexed {
    #[inline]
    pub(crate) fn push_result_token(
        &mut self,
        result_token: Result<crate::Token, crate::Error>,
    ) -> tombi_text::Span {
        match result_token {
            Ok(token) => {
                let span = token.span();
                self.tokens.push(token);
                span
            }
            Err(error) => {
                let span = error.span();
                self.tokens.push(crate::Token::new(INVALID_TOKEN, span));
                self.errors.push(error);
                span
            }
        }
    }
}
