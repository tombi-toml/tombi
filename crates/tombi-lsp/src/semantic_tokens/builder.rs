use tombi_ast_syntax::AstToken;
use tower_lsp_server::gen_lsp_types::SemanticToken;

use super::token_type::TokenType;
use crate::initialization_options::SemanticTokensSettings;

pub struct SemanticTokensBuilder<'a> {
    tokens: Vec<SemanticToken>,
    last_start: tombi_text::Position,
    line_index: &'a tombi_text::LineIndex<'a>,
    /// Tokens are added in document order, so their positions are converted by a cursor.
    cursor: tombi_text::LineIndexCursor<'a, 'a>,
    encoding: tombi_text::EncodingKind,
    /// Whether the client accepts a token that spans multiple lines.
    multiline_token_support: bool,
    settings: SemanticTokensSettings,
    pub text_document_uri: tombi_uri::Uri,
}

impl<'a> SemanticTokensBuilder<'a> {
    pub fn new(
        text_document_uri: tombi_uri::Uri,
        line_index: &'a tombi_text::LineIndex<'a>,
        encoding: tombi_text::EncodingKind,
        multiline_token_support: bool,
        settings: SemanticTokensSettings,
    ) -> Self {
        Self {
            tokens: Vec::new(),
            last_start: tombi_text::Position::default(),
            line_index,
            cursor: line_index.cursor(encoding),
            encoding,
            multiline_token_support,
            settings,
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

    /// Adds a string value token, unless the client disabled string tokens.
    pub fn add_string_token(&mut self, span: tombi_text::Span) {
        if self.settings.strings.enabled {
            self.add_token(TokenType::STRING, span);
        }
    }

    /// Adds an ordinary comment token, unless the client disabled comment tokens.
    pub fn add_comment_token(&mut self, span: tombi_text::Span) {
        if self.settings.comments.enabled {
            self.add_token(TokenType::COMMENT, span);
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
        if !self.settings.comments.enabled {
            // Only the directive keyword is kept; the rest of the comment is left uncolored.
            let (delta_line, delta_start) =
                delta_line_and_start(self.last_start, directive_span.start);
            self.last_start = directive_span.start;
            self.tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length: directive_span.end.column - directive_span.start.column,
                token_type: TokenType::KEYWORD as u32,
                token_modifiers_bitset: 0,
            });
            return;
        }

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

    /// Returns the absolute `(token_type, line, column, length)` of every token.
    fn tokens(
        source: &str,
        encoding: tombi_text::EncodingKind,
        multiline_token_support: bool,
        settings: SemanticTokensSettings,
    ) -> Vec<(u32, u32, u32, u32)> {
        let parsed = tombi_parser::parse(source);
        let mut builder = SemanticTokensBuilder::new(
            tombi_uri::Uri::from_str("file:///test.toml").unwrap(),
            parsed.line_index(),
            encoding,
            multiline_token_support,
            settings,
        );
        parsed.root().append_semantic_tokens(&mut builder);

        let mut line = 0;
        let mut column = 0;
        builder
            .build()
            .into_iter()
            .map(|token| {
                line += token.delta_line;
                column = if token.delta_line == 0 {
                    column + token.delta_start
                } else {
                    token.delta_start
                };
                (token.token_type, line, column, token.length)
            })
            .collect()
    }

    /// Returns the absolute `(line, column, length)` of every string token.
    fn string_tokens(
        source: &str,
        encoding: tombi_text::EncodingKind,
        multiline_token_support: bool,
    ) -> Vec<(u32, u32, u32)> {
        tokens(
            source,
            encoding,
            multiline_token_support,
            Default::default(),
        )
        .into_iter()
        .filter_map(|(token_type, line, column, length)| {
            (token_type == TokenType::STRING as u32).then_some((line, column, length))
        })
        .collect()
    }

    fn settings(strings: bool, comments: bool) -> SemanticTokensSettings {
        let mut settings = SemanticTokensSettings::default();
        settings.strings.enabled = strings;
        settings.comments.enabled = comments;
        settings
    }

    fn utf16_tokens(source: &str, settings: SemanticTokensSettings) -> Vec<(u32, u32, u32, u32)> {
        tokens(source, tombi_text::EncodingKind::Utf16, true, settings)
    }

    const STRING: u32 = TokenType::STRING as u32;
    const COMMENT: u32 = TokenType::COMMENT as u32;
    const KEYWORD: u32 = TokenType::KEYWORD as u32;
    const OPERATOR: u32 = TokenType::OPERATOR as u32;
    const TABLE: u32 = TokenType::TABLE as u32;
    const KEY: u32 = TokenType::KEY as u32;
    const NUMBER: u32 = TokenType::NUMBER as u32;

    const MIXED_SOURCE: &str =
        "# lead\nkey = \"v\" # trail\n\"quoted.key\" = 1\nm = \"\"\"a\nb\"\"\"\n";

    #[test]
    fn everything_is_sent_by_default() {
        assert_eq!(
            utf16_tokens(MIXED_SOURCE, settings(true, true)),
            [
                (COMMENT, 0, 0, 6),
                (KEY, 1, 0, 3),
                (OPERATOR, 1, 4, 1),
                (STRING, 1, 6, 3),
                (COMMENT, 1, 10, 7),
                (KEY, 2, 0, 12),
                (OPERATOR, 2, 13, 1),
                (NUMBER, 2, 15, 1),
                (KEY, 3, 0, 1),
                (OPERATOR, 3, 2, 1),
                (STRING, 3, 4, 9),
            ]
        );
    }

    #[test]
    fn strings_off_keeps_keys_and_other_tokens() {
        // Quoted keys are `KEY`, and every multi-line string is dropped.
        assert_eq!(
            utf16_tokens(MIXED_SOURCE, settings(false, true)),
            [
                (COMMENT, 0, 0, 6),
                (KEY, 1, 0, 3),
                (OPERATOR, 1, 4, 1),
                (COMMENT, 1, 10, 7),
                (KEY, 2, 0, 12),
                (OPERATOR, 2, 13, 1),
                (NUMBER, 2, 15, 1),
                (KEY, 3, 0, 1),
                (OPERATOR, 3, 2, 1),
            ]
        );
        assert!(
            tokens(
                MIXED_SOURCE,
                tombi_text::EncodingKind::Utf16,
                false,
                settings(false, true)
            )
            .iter()
            .all(|token| token.0 != STRING)
        );
    }

    #[test]
    fn comments_off_drops_leading_and_trailing_comments() {
        assert_eq!(
            utf16_tokens(MIXED_SOURCE, settings(true, false)),
            [
                (KEY, 1, 0, 3),
                (OPERATOR, 1, 4, 1),
                (STRING, 1, 6, 3),
                (KEY, 2, 0, 12),
                (OPERATOR, 2, 13, 1),
                (NUMBER, 2, 15, 1),
                (KEY, 3, 0, 1),
                (OPERATOR, 3, 2, 1),
                (STRING, 3, 4, 9),
            ]
        );
    }

    #[test]
    fn comments_off_drops_dangling_comments() {
        let source = "# root\n\n[a]\n# dangling\nk = 1\n";
        assert_eq!(
            utf16_tokens(source, settings(true, false)),
            [
                (OPERATOR, 2, 0, 1),
                (TABLE, 2, 1, 1),
                (OPERATOR, 2, 2, 1),
                (KEY, 4, 0, 1),
                (OPERATOR, 4, 2, 1),
                (NUMBER, 4, 4, 1),
            ]
        );
    }

    #[test]
    fn both_off() {
        assert_eq!(
            utf16_tokens(MIXED_SOURCE, settings(false, false)),
            [
                (KEY, 1, 0, 3),
                (OPERATOR, 1, 4, 1),
                (KEY, 2, 0, 12),
                (OPERATOR, 2, 13, 1),
                (NUMBER, 2, 15, 1),
                (KEY, 3, 0, 1),
                (OPERATOR, 3, 2, 1),
            ]
        );
    }

    #[test]
    fn directives_and_regions_survive_comments_off() {
        let source = "#:tombi toml-version = \"v1.0.0\"\n\n# region deps\n[a]\nv = 1 # tombi: lint.rules = {}\n# endregion deps\n# plain\nz = \"s\"\n";
        let all = utf16_tokens(source, settings(true, true));
        assert!(all.contains(&(COMMENT, 2, 0, 2)));
        // Only the keyword of each directive remains, at the same positions as by default,
        // and the deltas stay consistent over the omitted tokens.
        assert_eq!(
            utf16_tokens(source, settings(true, false)),
            [
                (KEYWORD, 0, 1, 6),
                (KEYWORD, 2, 2, 6),
                (OPERATOR, 3, 0, 1),
                (TABLE, 3, 1, 1),
                (OPERATOR, 3, 2, 1),
                (KEY, 4, 0, 1),
                (OPERATOR, 4, 2, 1),
                (NUMBER, 4, 4, 1),
                (KEYWORD, 4, 8, 6),
                (KEYWORD, 5, 2, 9),
                (KEY, 7, 0, 1),
                (OPERATOR, 7, 2, 1),
                (STRING, 7, 4, 3),
            ]
        );
        let keywords = |settings| {
            utf16_tokens(source, settings)
                .into_iter()
                .filter(|token| token.0 == KEYWORD)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            keywords(settings(true, true)),
            keywords(settings(true, false))
        );
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
