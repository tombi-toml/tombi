use tombi_ast::AstToken;
use tombi_text::FromLsp;
use tower_lsp::lsp_types::SemanticToken;
use unicode_segmentation::UnicodeSegmentation;

use super::token_type::TokenType;

pub struct SemanticTokensBuilder<'a> {
    tokens: Vec<SemanticToken>,
    last_range: tombi_text::Range,
    line_index: &'a tombi_text::LineIndex<'a>,
    pub text_document_uri: tombi_uri::Uri,
}

impl<'a> SemanticTokensBuilder<'a> {
    pub fn new(
        text_document_uri: tombi_uri::Uri,
        line_index: &'a tombi_text::LineIndex<'a>,
    ) -> Self {
        Self {
            tokens: Vec::new(),
            last_range: tombi_text::Range::default(),
            line_index,
            text_document_uri,
        }
    }

    pub fn add_token(&mut self, token_type: TokenType, elem: tombi_syntax::SyntaxElement) {
        let range: tombi_text::Range = elem.range();
        let (delta_line, delta_start) =
            delta_line_and_start(self.last_range.start, range.start, self.line_index);

        #[allow(clippy::cast_possible_truncation)]
        self.tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: token_length(range, self.line_index),
            token_type: token_type as u32,
            token_modifiers_bitset: 0,
        });

        self.last_range = range;
    }

    pub fn add_comment_directive(
        &mut self,
        comment: impl AsRef<tombi_ast::Comment>,
        directive_range: tombi_text::Range,
    ) {
        let comment_range = comment.as_ref().syntax().range();
        let directive_range =
            tower_lsp::lsp_types::Range::from_lsp(directive_range, self.line_index);
        let (delta_line, delta_start) =
            delta_line_and_start(self.last_range.start, comment_range.start, self.line_index);

        self.last_range = comment_range;

        let comment_range = tower_lsp::lsp_types::Range::from_lsp(comment_range, self.line_index);
        let prefix_len = directive_range.start.character - comment_range.start.character;
        let directive_len = directive_range.end.character - directive_range.start.character;
        let content_len = comment_range.end.character - directive_range.end.character;

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
}

fn delta_line_and_start(
    start: tombi_text::Position,
    end: tombi_text::Position,
    line_index: &tombi_text::LineIndex,
) -> (u32, u32) {
    let delta_start = tower_lsp::lsp_types::Position::from_lsp(start, line_index);
    let delta_end = tower_lsp::lsp_types::Position::from_lsp(end, line_index);
    let line = delta_end.line - delta_start.line;
    let start = if line == 0 {
        delta_end.character - delta_start.character
    } else {
        delta_end.character
    };
    (line, start)
}

fn token_length(range: tombi_text::Range, line_index: &tombi_text::LineIndex) -> u32 {
    let encoding_kind = line_index.encoding_kind;

    if range.start.line == range.end.line {
        let Some(line_text) = line_index.line_text(range.start.line) else {
            tracing::error!("line_text is None. line: {}", range.start.line);
            return 0;
        };
        let line_text_graphemes = line_text.graphemes(true);
        line_text_graphemes
            .skip(range.start.column as usize)
            .take((range.end.column - range.start.column) as usize)
            .fold(0, |acc, char| acc + encoding_kind.measure(char))
    } else {
        (range.start.line..=range.end.line).fold(0, |acc, line| {
            acc + line_index
                .line_text(line)
                .map(|line_text| {
                    let line_text_graphemes = line_text.graphemes(true);
                    let skip_count = if line == range.start.line {
                        range.start.column as usize
                    } else {
                        0
                    };
                    let take_count = if line == range.end.line {
                        if range.end.line == range.start.line {
                            (range.end.column - range.start.column) as usize
                        } else {
                            range.end.column as usize
                        }
                    } else {
                        line_text_graphemes.size_hint().1.unwrap()
                    };
                    line_text_graphemes
                        .skip(skip_count)
                        .take(take_count)
                        .fold(0, |acc, char| acc + encoding_kind.measure(char))
                })
                .unwrap_or_default()
        })
    }
}
