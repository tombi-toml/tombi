use tombi_ast::AstToken;
use tombi_text::{FromLsp, IntoLsp};
use tower_lsp::lsp_types::SemanticToken;

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
        let range = elem.range();

        let relative = relative_range(range, self.last_range, self.line_index);

        #[allow(clippy::cast_possible_truncation)]
        self.tokens.push(SemanticToken {
            delta_line: relative.start.line as u32,
            delta_start: relative.start.character as u32,
            length: elem.span().len(),
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

        let relative = relative_range(comment_range, self.last_range, self.line_index);
        let directive_range =
            tower_lsp::lsp_types::Range::from_lsp(directive_range, self.line_index);
        self.last_range = comment_range;
        let comment_range = tower_lsp::lsp_types::Range::from_lsp(comment_range, self.line_index);
        let prefix_len = directive_range.start.character - comment_range.start.character;
        let directive_len = directive_range.end.character - directive_range.start.character;

        self.tokens.push(SemanticToken {
            delta_line: relative.start.line as u32,
            delta_start: relative.start.character as u32,
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
            length: (comment_range.end.character - directive_range.end.character),
            token_type: TokenType::COMMENT as u32,
            token_modifiers_bitset: 0,
        });
    }

    pub fn build(self) -> Vec<SemanticToken> {
        self.tokens
    }
}

fn relative_range(
    from: tombi_text::Range,
    to: tombi_text::Range,
    line_index: &tombi_text::LineIndex,
) -> tower_lsp::lsp_types::Range {
    let line_diff = from.end.line - from.start.line;
    let start = from.start - to.start;

    let end = if line_diff == 0 {
        tombi_text::Position::new(
            start.line,
            start.column + from.end.column - from.start.column,
        )
    } else {
        tombi_text::Position::new(start.line + line_diff, from.end.column)
    };

    tombi_text::Range::new((start.line, start.column).into(), end).into_lsp(line_index)
}
