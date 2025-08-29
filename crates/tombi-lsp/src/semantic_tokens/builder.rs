use tombi_ast::AstToken;
use tower_lsp::lsp_types::SemanticToken;

use super::token_type::TokenType;

pub struct SemanticTokensBuilder {
    tokens: Vec<SemanticToken>,
    last_range: tombi_text::Range,
    pub text_document_uri: tombi_uri::Uri,
}

impl SemanticTokensBuilder {
    pub fn new(text_document_uri: tombi_uri::Uri) -> Self {
        Self {
            tokens: Vec::new(),
            last_range: tombi_text::Range::default(),
            text_document_uri,
        }
    }

    pub fn add_token(&mut self, token_type: TokenType, elem: tombi_syntax::SyntaxElement) {
        let range = elem.range();

        let relative = relative_range(range, self.last_range);

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
        directive_range: &tombi_text::Range,
    ) {
        let comment_range = comment.as_ref().syntax().range();

        let relative = relative_range(comment_range, self.last_range);
        let prefix_len = directive_range.start.column - comment_range.start.column;
        let directive_len = directive_range.end.column - directive_range.start.column;

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
            length: (comment_range.end.column - directive_range.end.column),
            token_type: TokenType::COMMENT as u32,
            token_modifiers_bitset: 0,
        });

        self.last_range = comment_range;
    }

    pub fn build(self) -> Vec<SemanticToken> {
        self.tokens
    }
}

fn relative_range(from: tombi_text::Range, to: tombi_text::Range) -> tower_lsp::lsp_types::Range {
    let line_diff = from.end.line - from.start.line;
    let start = from.start - to.start;
    let start = tower_lsp::lsp_types::Position {
        line: start.line,
        character: start.column,
    };

    let end = if line_diff == 0 {
        tower_lsp::lsp_types::Position {
            line: start.line,
            character: start.character + from.end.column - from.start.column,
        }
    } else {
        tower_lsp::lsp_types::Position {
            line: start.line + line_diff,
            character: from.end.column,
        }
    };

    tower_lsp::lsp_types::Range { start, end }
}
