use ast::AstNode;
use parser::SyntaxNode;
use text_position::TextPosition;
use tower_lsp::lsp_types::{
    Position, Range, SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensParams,
    SemanticTokensResult,
};

use crate::{server::backend::Backend, toml};

#[derive(Debug, Copy, Clone)]
#[repr(u32)]
pub enum TokenType {
    STRUCT,
    STRING,
    NUMBER,
    KEYWORD,
    VARIABLE,
    REGEXP,
    COMMENT,
}

impl TokenType {
    pub const LEGEND: &'static [SemanticTokenType] = &[
        SemanticTokenType::STRUCT,
        SemanticTokenType::STRING,
        SemanticTokenType::NUMBER,
        SemanticTokenType::KEYWORD,
        SemanticTokenType::VARIABLE,
        SemanticTokenType::REGEXP,
        SemanticTokenType::COMMENT,
    ];
}

pub async fn handle_semantic_tokens_full(
    _backend: &Backend,
    SemanticTokensParams { text_document, .. }: SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>, tower_lsp::jsonrpc::Error> {
    let source = toml::try_load(&text_document.uri)?;

    let p = parser::parse(&source);
    let Some(ast) = ast::Root::cast(p.into_syntax_node()) else {
        return Ok(None);
    };

    let mut tokens_builder = SemanticTokensBuilder::new(&source);
    ast.append_semantic_tokens(&mut tokens_builder);
    let tokens = tokens_builder.build();

    tracing::info!("SemanticTokens: {tokens:#?}");

    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })))
}

trait AppendSemanticTokens {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder);
}

impl AppendSemanticTokens for ast::Root {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for item in self.items() {
            item.append_semantic_tokens(builder);
        }
    }
}

impl AppendSemanticTokens for ast::RootItem {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        match self {
            Self::Table(table) => table.append_semantic_tokens(builder),
            Self::ArrayOfTable(array) => array.append_semantic_tokens(builder),
            Self::KeyValue(key_value) => key_value.append_semantic_tokens(builder),
        }
    }
}

impl AppendSemanticTokens for ast::Table {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        if let Some(header) = self.header() {
            for key in header.keys() {
                builder.add_token(TokenType::STRUCT, key.syntax());
            }
        }

        for entry in self.key_values() {
            entry.append_semantic_tokens(builder);
        }
    }
}

impl AppendSemanticTokens for ast::ArrayOfTable {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        if let Some(header) = self.header() {
            for key in header.keys() {
                builder.add_token(TokenType::STRUCT, key.syntax());
            }
        }

        for table in self.key_values() {
            table.append_semantic_tokens(builder);
        }
    }
}

impl AppendSemanticTokens for ast::KeyValue {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        self.keys().map(|key| key.append_semantic_tokens(builder));
        self.value()
            .map(|value| value.append_semantic_tokens(builder));
    }
}

impl AppendSemanticTokens for ast::Keys {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for key in self.keys() {
            key.append_semantic_tokens(builder);
        }
    }
}

impl AppendSemanticTokens for ast::Key {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        builder.add_token(TokenType::VARIABLE, self.syntax());
    }
}

impl AppendSemanticTokens for ast::Value {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        match self {
            Self::BasicString(_)
            | Self::LiteralString(_)
            | Self::MultiLineBasicString(_)
            | Self::MultiLineLiteralString(_) => {
                builder.add_token(TokenType::STRING, self.syntax())
            }
            Self::IntegerBin(_)
            | Self::IntegerOct(_)
            | Self::IntegerDec(_)
            | Self::IntegerHex(_)
            | Self::Float(_) => builder.add_token(TokenType::NUMBER, self.syntax()),
            Self::Boolean(_) => builder.add_token(TokenType::KEYWORD, self.syntax()),
            Self::OffsetDateTime(_)
            | Self::LocalDateTime(_)
            | Self::LocalDate(_)
            | Self::LocalTime(_) => builder.add_token(TokenType::REGEXP, self.syntax()),
            Self::Array(array) => array.append_semantic_tokens(builder),
            Self::InlineTable(inline_table) => inline_table.append_semantic_tokens(builder),
        }
    }
}

impl AppendSemanticTokens for ast::Array {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for value in self.elements() {
            value.append_semantic_tokens(builder);
        }
    }
}

impl AppendSemanticTokens for ast::InlineTable {
    fn append_semantic_tokens(&self, builder: &mut SemanticTokensBuilder) {
        for entry in self.elements() {
            entry.append_semantic_tokens(builder);
        }
    }
}

struct SemanticTokensBuilder<'a> {
    tokens: Vec<SemanticToken>,
    last_range: Range,
    source: &'a str,
}

impl<'a> SemanticTokensBuilder<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            tokens: Vec::new(),
            last_range: Range::default(),
            source,
        }
    }

    fn add_token(&mut self, token_type: TokenType, node: &SyntaxNode) {
        let range = Range::new(
            TextPosition::from_source(self.source, node.text_range().start()).into(),
            TextPosition::from_source(self.source, node.text_range().end()).into(),
        );

        let relative = relative_range(range, self.last_range);

        #[allow(clippy::cast_possible_truncation)]
        self.tokens.push(SemanticToken {
            delta_line: relative.start.line as u32,
            delta_start: relative.start.character as u32,
            length: (relative.end.character - relative.start.character) as u32,
            token_type: token_type as u32,
            token_modifiers_bitset: 0,
        });

        self.last_range = range;
    }

    fn build(self) -> Vec<SemanticToken> {
        self.tokens
    }
}

pub fn relative_position(position: Position, to: Position) -> Position {
    if position.line == to.line {
        Position {
            line: 0,
            character: position.character - to.character,
        }
    } else {
        Position {
            line: position.line - to.line,
            character: position.character,
        }
    }
}

pub fn relative_range(range: Range, to: Range) -> Range {
    let line_diff = range.end.line - range.start.line;
    let start = relative_position(range.start, to.start);

    let end = if line_diff == 0 {
        Position {
            line: start.line,
            character: start.character + range.end.character - range.start.character,
        }
    } else {
        Position {
            line: start.line + line_diff,
            character: range.end.character,
        }
    };

    Range { start, end }
}