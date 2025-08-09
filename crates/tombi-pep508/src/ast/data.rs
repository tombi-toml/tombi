use crate::SyntaxKind;

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
