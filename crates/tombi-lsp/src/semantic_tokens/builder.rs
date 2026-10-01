use tombi_ast_syntax::AstToken;
use tower_lsp::lsp_types::SemanticToken;

use super::token_type::TokenType;

pub struct SemanticTokensBuilder<'a> {
    tokens: Vec<SemanticToken>,
    last_start: tombi_text::Position,
    line_index: &'a tombi_text::LineIndex<'a>,
    /// Tokens are added in document order, so their positions are converted by a cursor.
    cursor: tombi_text::LineIndexCursor<'a, 'a>,
    encoding: tombi_text::EncodingKind,
    pub text_document_uri: tombi_uri::Uri,
}

impl<'a> SemanticTokensBuilder<'a> {
    pub fn new(
        text_document_uri: tombi_uri::Uri,
        line_index: &'a tombi_text::LineIndex<'a>,
        encoding: tombi_text::EncodingKind,
    ) -> Self {
        Self {
            tokens: Vec::new(),
            last_start: tombi_text::Position::default(),
            line_index,
            cursor: line_index.cursor(encoding),
            encoding,
            text_document_uri,
        }
    }

    pub fn add_token(&mut self, token_type: TokenType, span: tombi_text::Span) {
        let range = self.cursor.range(span);
        let (delta_line, delta_start) = delta_line_and_start(self.last_start, range.start);

        self.tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: self.token_length(span, range),
            token_type: token_type as u32,
            token_modifiers_bitset: 0,
        });

        self.last_start = range.start;
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

        self.tokens.push(SemanticToken {
            delta_line: 0,
            delta_start: directive_len,
            length: content_len,
            token_type: TokenType::COMMENT as u32,
            token_modifiers_bitset: 0,
        });
    }

    pub fn build(self) -> Vec<SemanticToken> {
        self.tokens
    }

    /// The length of a token, without the line breaks of a multi-line token.
    fn token_length(&self, span: tombi_text::Span, range: tombi_text::Range) -> u32 {
        if range.start.line == range.end.line {
            return range.end.column - range.start.column;
        }
        (range.start.line..=range.end.line)
            .filter_map(|line| self.line_index.line_span(line)?.intersect(span))
            .map(|part| self.encoding.measure(&self.line_index.text()[part]))
            .sum()
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
