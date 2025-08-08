#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pep508Language {}

impl tombi_rg_tree::Language for Pep508Language {
    type Kind = crate::SyntaxKind;

    fn kind_from_raw(raw: tombi_rg_tree::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= crate::SyntaxKind::__LAST as u16);
        unsafe { std::mem::transmute::<u16, crate::SyntaxKind>(raw.0) }
    }

    fn kind_to_raw(kind: Self::Kind) -> tombi_rg_tree::SyntaxKind {
        kind.into()
    }
}