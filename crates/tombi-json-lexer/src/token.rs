use tombi_json_syntax::SyntaxKind;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Token {
    kind: SyntaxKind,
    contains_escape: bool,
    span: tombi_text::Span,
    range: tombi_text::Range,
}

impl Token {
    pub fn new(kind: SyntaxKind, (span, range): (tombi_text::Span, tombi_text::Range)) -> Self {
        Self {
            kind,
            contains_escape: false,
            span,
            range,
        }
    }

    pub fn new_string(
        contains_escape: bool,
        (span, range): (tombi_text::Span, tombi_text::Range),
    ) -> Self {
        Self {
            kind: SyntaxKind::STRING,
            contains_escape,
            span,
            range,
        }
    }

    pub const fn eof() -> Self {
        Self {
            kind: SyntaxKind::EOF,
            contains_escape: false,
            span: tombi_text::Span::MAX,
            range: tombi_text::Range::MAX,
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

    #[inline]
    pub fn range(&self) -> tombi_text::Range {
        self.range
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} @{} @{} (contains_escape: {})",
            self.kind, self.span, self.range, self.contains_escape
        )
    }
}
