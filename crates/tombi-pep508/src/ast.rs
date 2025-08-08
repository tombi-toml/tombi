use crate::SyntaxKind;
use tombi_rg_tree::Language;

// Data structures for parsed PEP 508 requirements
#[derive(Debug, Clone, PartialEq)]
pub struct Pep508Requirement {
    pub name: String,
    pub extras: Vec<String>,
    pub version_spec: Option<VersionSpec>,
    pub marker: Option<MarkerExpression>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionSpec {
    pub clauses: Vec<VersionClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VersionClause {
    pub operator: VersionOperator,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VersionOperator {
    Equal,            // ==
    NotEqual,         // !=
    LessThanEqual,    // <=
    GreaterThanEqual, // >=
    LessThan,         // <
    GreaterThan,      // >
    Compatible,       // ~=
    ArbitraryEqual,   // ===
}

impl From<SyntaxKind> for VersionOperator {
    fn from(kind: SyntaxKind) -> Self {
        match kind {
            SyntaxKind::EQ_EQ => VersionOperator::Equal,
            SyntaxKind::NOT_EQ => VersionOperator::NotEqual,
            SyntaxKind::LTE => VersionOperator::LessThanEqual,
            SyntaxKind::GTE => VersionOperator::GreaterThanEqual,
            SyntaxKind::LT => VersionOperator::LessThan,
            SyntaxKind::GT => VersionOperator::GreaterThan,
            SyntaxKind::TILDE_EQ => VersionOperator::Compatible,
            SyntaxKind::EQ_EQ_EQ => VersionOperator::ArbitraryEqual,
            _ => panic!("Invalid version operator kind: {:?}", kind),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkerExpression {
    pub expression: String,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

// Re-export AST types
pub use crate::language::Pep508Language;
pub type SyntaxNode = tombi_rg_tree::RedNode<Pep508Language>;
pub type SyntaxToken = tombi_rg_tree::RedToken<Pep508Language>;
pub type SyntaxElement = tombi_rg_tree::RedElement<Pep508Language>;
pub type SyntaxNodePtr = tombi_rg_tree::RedNodePtr<Pep508Language>;
pub type SyntaxNodeChildren = tombi_rg_tree::RedNodeChildren<Pep508Language>;
pub type SyntaxElementChildren = tombi_rg_tree::RedElementChildren<Pep508Language>;
pub type PreorderWithTokens = tombi_rg_tree::RedPreorderWithTokens<Pep508Language>;

/// Parse a PEP 508 requirement string into an AST
pub fn parse(source: &str) -> (SyntaxNode, Vec<crate::Error>) {
    let parser = crate::parser::Parser::new(source);
    parser.parse_ast()
}

/// Syntax tree builder for PEP 508 AST
#[derive(Debug)]
pub struct SyntaxTreeBuilder<E> {
    inner: tombi_rg_tree::GreenNodeBuilder<'static>,
    errors: Vec<E>,
}

impl<E> SyntaxTreeBuilder<E> {
    pub fn finish(self) -> (tombi_rg_tree::GreenNode, Vec<E>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn token(&mut self, kind: crate::SyntaxKind, text: &str) {
        let kind = Pep508Language::kind_to_raw(kind);
        self.inner.token(kind, text);
    }

    pub fn start_node(&mut self, kind: crate::SyntaxKind) {
        let kind = Pep508Language::kind_to_raw(kind);
        self.inner.start_node(kind);
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node();
    }

    pub fn error(&mut self, error: E) {
        self.errors.push(error);
    }
}

impl<E> Default for SyntaxTreeBuilder<E> {
    fn default() -> SyntaxTreeBuilder<E> {
        SyntaxTreeBuilder {
            inner: tombi_rg_tree::GreenNodeBuilder::new(),
            errors: Vec::new(),
        }
    }
}