use tombi_json_syntax::SyntaxKind;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Token {
    kind: SyntaxKind,
    contains_escape: bool,
    span: tombi_text::Span,
}

impl Token {
    pub fn new(kind: SyntaxKind, span: tombi_text::Span) -> Self {
        Self {
            kind,
            contains_escape: false,
            span,
        }
    }

    pub fn new_string(contains_escape: bool, span: tombi_text::Span) -> Self {
        Self {
            kind: SyntaxKind::STRING,
            contains_escape,
            span,
        }
    }

    pub const fn eof() -> Self {
        Self {
            kind: SyntaxKind::EOF,
            contains_escape: false,
            span: tombi_text::Span::MAX,
        }
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.kind == SyntaxKind::EOF
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    #[inline]
    pub fn contains_escape(&self) -> bool {
        self.contains_escape
    }

    #[inline]
    pub fn span(&self) -> tombi_text::Span {
        self.span
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} @{} (contains_escape: {})",
            self.kind, self.span, self.contains_escape
        )
    }
}
