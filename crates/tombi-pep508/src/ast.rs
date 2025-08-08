use crate::SyntaxKind;
use std::marker::PhantomData;
use tombi_rg_tree::Language;

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

// ============= AstNode and AstToken traits =============

pub trait AstNode: std::fmt::Debug {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;

    fn range(&self) -> tombi_text::Range {
        self.syntax().range()
    }

    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_for_update()).unwrap()
    }
}

pub trait AstToken {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: SyntaxNodeChildren,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

// ============= AST Node implementations =============

/// Root node of a PEP 508 requirement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Root {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for Root {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::ROOT
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl Root {
    pub fn requirement(&self) -> Option<Requirement> {
        self.syntax.children().find_map(Requirement::cast)
    }
}

/// A PEP 508 requirement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Requirement {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for Requirement {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::REQUIREMENT
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl Requirement {
    pub fn package_name(&self) -> Option<PackageName> {
        self.syntax.children().find_map(PackageName::cast)
    }

    pub fn extras_list(&self) -> Option<ExtrasList> {
        self.syntax.children().find_map(ExtrasList::cast)
    }

    pub fn version_spec(&self) -> Option<VersionSpecNode> {
        self.syntax.children().find_map(VersionSpecNode::cast)
    }

    pub fn url_spec(&self) -> Option<UrlSpec> {
        self.syntax.children().find_map(UrlSpec::cast)
    }

    pub fn marker(&self) -> Option<MarkerExpr> {
        self.syntax.children().find_map(MarkerExpr::cast)
    }
}

/// Package name node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageName {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for PackageName {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::PACKAGE_NAME
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl PackageName {
    pub fn identifier(&self) -> Option<Identifier> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find_map(Identifier::cast)
    }

    pub fn name(&self) -> Option<String> {
        self.identifier().map(|id| id.text().to_string())
    }
}

/// Extras list node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtrasList {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for ExtrasList {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::EXTRAS_LIST
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl ExtrasList {
    pub fn extras(&self) -> impl Iterator<Item = String> + '_ {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .filter_map(Identifier::cast)
            .map(|id| id.text().to_string())
    }

    pub fn bracket_start(&self) -> Option<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::BRACKET_START)
    }

    pub fn bracket_end(&self) -> Option<SyntaxToken> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::BRACKET_END)
    }
}

/// Version specification node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionSpecNode {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for VersionSpecNode {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::VERSION_SPEC
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl VersionSpecNode {
    pub fn clauses(&self) -> AstChildren<VersionClauseNode> {
        AstChildren::new(&self.syntax)
    }
}

/// Version clause node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionClauseNode {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for VersionClauseNode {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::VERSION_CLAUSE
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl VersionClauseNode {
    pub fn operator(&self) -> Option<VersionOperator> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind().is_version_operator())
            .map(|t| VersionOperator::from(t.kind()))
    }

    pub fn version(&self) -> Option<String> {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .find(|t| t.kind() == SyntaxKind::VERSION_STRING)
            .map(|t| t.text().to_string())
    }
}

/// URL specification node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UrlSpec {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for UrlSpec {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::URL_SPEC
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl UrlSpec {
    pub fn url(&self) -> String {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .filter(|t| !t.kind().is_trivia())
            .map(|t| t.text().to_string())
            .collect()
    }
}

/// Marker expression node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarkerExpr {
    pub(crate) syntax: SyntaxNode,
}

impl AstNode for MarkerExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::MARKER_EXPR
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

impl MarkerExpr {
    pub fn expression(&self) -> String {
        self.syntax
            .children_with_tokens()
            .filter_map(|e| e.into_token())
            .map(|t| {
                if t.kind() == SyntaxKind::WHITESPACE {
                    " ".to_string()
                } else {
                    t.text().to_string()
                }
            })
            .collect::<String>()
            .trim()
            .to_string()
    }
}

// ============= AST Token implementations =============

/// Identifier token
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub(crate) syntax: SyntaxToken,
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.syntax, f)
    }
}

impl AstToken for Identifier {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::IDENTIFIER
    }

    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

/// Version string token
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VersionString {
    pub(crate) syntax: SyntaxToken,
}

impl std::fmt::Display for VersionString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.syntax, f)
    }
}

impl AstToken for VersionString {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SyntaxKind::VERSION_STRING
    }

    fn cast(syntax: SyntaxToken) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }

    fn syntax(&self) -> &SyntaxToken {
        &self.syntax
    }
}

// ============= Data structures for compatibility =============

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