use tombi_ast_syntax::AstToken;
use tower_lsp_server::gen_lsp_types::SemanticToken;

use super::token_type::TokenType;

pub struct SemanticTokensBuilder<'a> {
    tokens: Vec<SemanticToken>,
    last_start: tombi_text::Position,
    line_index: &'a tombi_text::LineIndex<'a>,
    /// Tokens are added in document order, so their positions are converted by a cursor.
    cursor: tombi_text::LineIndexCursor<'a, 'a>,
    encoding: tombi_text::EncodingKind,
    /// Whether the client accepts a token that spans multiple lines.
    multiline_token_support: bool,
    pub text_document_uri: tombi_uri::Uri,
}

impl<'a> SemanticTokensBuilder<'a> {
    pub fn new(
        text_document_uri: tombi_uri::Uri,
        line_index: &'a tombi_text::LineIndex<'a>,
        encoding: tombi_text::EncodingKind,
        multiline_token_support: bool,
    ) -> Self {
        Self {
            tokens: Vec::new(),
            last_start: tombi_text::Position::default(),
            line_index,
            cursor: line_index.cursor(encoding),
            encoding,
            multiline_token_support,
            text_document_uri,
        }
    }

    pub fn add_token(&mut self, token_type: TokenType, span: tombi_text::Span) {
        let token_type = token_type as u32;
        let range = self.cursor.range(span);

        if range.start.line == range.end.line || self.multiline_token_support {
            // The length of a multi-line token counts its line breaks,
            // because the length is that of the token text.
            let length = self.encoding.measure(&self.line_index.text()[span]);
            self.push_token(token_type, range.start, length);
            return;
        }

        // The client cannot render a token that runs past the end of a line,
        // so the token is split into one token per line.
        for line in range.start.line..=range.end.line {
            let Some(part) = self
                .line_index
                .line_span(line)
                .and_then(|line_span| line_span.intersect(span))
            else {
                continue;
            };
            let length = self.encoding.measure(&self.line_index.text()[part]);
            if length == 0 {
                continue;
            }
            let column = if line == range.start.line {
                range.start.column
            } else {
                0
            };
            self.push_token(token_type, tombi_text::Position::new(line, column), length);
        }
    }

    fn push_token(&mut self, token_type: u32, start: tombi_text::Position, length: u32) {
        let (delta_line, delta_start) = delta_line_and_start(self.last_start, start);

        self.tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type,
            token_modifiers_bitset: 0,
        });

        self.last_start = start;
    }

    pub fn add_comment_directive<'c>(
        &mut self,
        comment: impl AsRef<tombi_ast_syntax::Comment<'c>>,
        directive_span: tombi_text::Span,
    ) {
        let comment_span = self.cursor.range(comment.as_ref().syntax().span());
        let directive_span = self.cursor.range(directive_span);
        let (delta_line, delta_start) = delta_line_and_start(self.last_start, comment_span.start);

        self.last_start = comment_span.start;

        let prefix_len = directive_span.start.column - comment_span.start.column;
        let directive_len = directive_span.end.column - directive_span.start.column;
        let content_len = comment_span.end.column - directive_span.end.column;

        self.tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: prefix_len,
            token_type: TokenType::COMMENT as u32,
            token_modifiers_bitset: 0,
        });

        self.tokens.push(SemanticToken {
            delta_line: 0,
            delta_start: prefix_len,
            length: directive_len,
            token_type: TokenType::KEYWORD as u32,
            token_modifiers_bitset: 0,
        });

        if content_len > 0 {
            self.tokens.push(SemanticToken {
                delta_line: 0,
                delta_start: directive_len,
                length: content_len,
                token_type: TokenType::COMMENT as u32,
                token_modifiers_bitset: 0,
            });
        }
    }

    pub fn build(self) -> Vec<SemanticToken> {
        self.tokens
    }
}

fn delta_line_and_start(start: tombi_text::Position, end: tombi_text::Position) -> (u32, u32) {
    let line = end.line - start.line;
    let start = if line == 0 {
        end.column - start.column
    } else {
        end.column
    };
    (line, start)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::super::{AppendSemanticTokens, TokenType};
    use super::*;

    /// Returns the absolute `(line, column, length)` of every string token.
    fn string_tokens(
        source: &str,
        encoding: tombi_text::EncodingKind,
        multiline_token_support: bool,
    ) -> Vec<(u32, u32, u32)> {
        let parsed = tombi_parser::parse(source);
        let mut builder = SemanticTokensBuilder::new(
            tombi_uri::Uri::from_str("file:///test.toml").unwrap(),
            parsed.line_index(),
            encoding,
            multiline_token_support,
        );
        parsed.root().append_semantic_tokens(&mut builder);

        let mut line = 0;
        let mut column = 0;
        builder
            .build()
            .into_iter()
            .filter_map(|token| {
                line += token.delta_line;
                column = if token.delta_line == 0 {
                    column + token.delta_start
                } else {
                    token.delta_start
                };
                (token.token_type == TokenType::STRING as u32).then_some((
                    line,
                    column,
                    token.length,
                ))
            })
            .collect()
    }

    #[test]
    fn multiline_basic_string_is_split_without_support() {
        let source = "a = \"\"\"abc\n\n  de\"\"\"\nb = 1\n";
        assert_eq!(
            string_tokens(source, tombi_text::EncodingKind::Utf16, false),
            [(0, 4, 6), (2, 0, 7)]
        );
    }

    #[test]
    fn multiline_literal_string_is_split_without_support() {
        let source = "a = '''abc\r\nde'''\n";
        assert_eq!(
            string_tokens(source, tombi_text::EncodingKind::Utf16, false),
            [(0, 4, 6), (1, 0, 5)]
        );
    }

    #[test]
    fn multiline_basic_string_is_one_token_with_support() {
        // The length counts the line breaks: `"""abc\n\n  de"""` is 15 units.
        let source = "a = \"\"\"abc\n\n  de\"\"\"\nb = 1\n";
        assert_eq!(
            string_tokens(source, tombi_text::EncodingKind::Utf16, true),
            [(0, 4, 15)]
        );
    }

    #[test]
    fn multiline_literal_string_is_one_token_with_support() {
        // `'''abc\r\nde'''` is 13 units, with a CRLF line break.
        let source = "a = '''abc\r\nde'''\n";
        assert_eq!(
            string_tokens(source, tombi_text::EncodingKind::Utf16, true),
            [(0, 4, 13)]
        );
    }

    #[test]
    fn multibyte_lengths_follow_the_encoding() {
        // "🦅" is 4 bytes in UTF-8, 2 units in UTF-16 and 1 in UTF-32.
        let source = "a = \"\"\"あ🦅\nい\"\"\"\n";
        for (encoding, expected) in [
            (
                tombi_text::EncodingKind::Utf8,
                [(0, 4, 3 + 3 + 4), (1, 0, 3 + 3)],
            ),
            (
                tombi_text::EncodingKind::Utf16,
                [(0, 4, 3 + 1 + 2), (1, 0, 1 + 3)],
            ),
            (
                tombi_text::EncodingKind::Utf32,
                [(0, 4, 3 + 1 + 1), (1, 0, 1 + 3)],
            ),
        ] {
            assert_eq!(string_tokens(source, encoding, false), expected);
        }
        assert_eq!(
            string_tokens(source, tombi_text::EncodingKind::Utf16, true),
            [(0, 4, 3 + 1 + 2 + 1 + 1 + 3)]
        );
    }
}
