// Submodules
pub mod builder;
pub mod data;
pub mod node;
pub mod token;
pub mod traits;

// Re-export main types
pub use builder::SyntaxTreeBuilder;
pub use data::{
    MarkerExpression, ParseError, Pep508Requirement, VersionClause, VersionOperator, VersionSpec,
};

// Re-export node types
pub use node::{
    ExtrasList, MarkerExpr, PackageName, Requirement, Root, UrlSpec, VersionClauseNode,
    VersionSpecNode,
};

// Re-export token types
pub use token::{Identifier, VersionString};

// Re-export trait types
pub use traits::{
    AstChildren, AstNode, AstToken, Pep508Language, PreorderWithTokens, SyntaxElement,
    SyntaxElementChildren, SyntaxNode, SyntaxNodeChildren, SyntaxNodePtr, SyntaxToken,
};

/// Parse a PEP 508 requirement string into an AST
pub fn parse(source: &str) -> (SyntaxNode, Vec<crate::Error>) {
    let parser = crate::parser::Parser::new(source);
    parser.parse_ast()
}
