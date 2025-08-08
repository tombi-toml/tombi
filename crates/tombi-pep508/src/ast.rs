// Submodules
pub mod builder;
pub mod data;
pub mod extras_list;
pub mod marker_expr;
pub mod package_name;
pub mod requirement;
pub mod root;
pub mod tokens;
pub mod traits;
pub mod url_spec;
pub mod version_spec;

// Re-export main types
pub use builder::SyntaxTreeBuilder;
pub use data::{
    MarkerExpression, ParseError, Pep508Requirement, VersionClause, VersionOperator, VersionSpec,
};
pub use extras_list::ExtrasList;
pub use marker_expr::MarkerExpr;
pub use package_name::PackageName;
pub use requirement::Requirement;
pub use root::Root;
pub use tokens::{Identifier, VersionString};
pub use traits::{
    AstChildren, AstNode, AstToken, Pep508Language, PreorderWithTokens, SyntaxElement,
    SyntaxElementChildren, SyntaxNode, SyntaxNodeChildren, SyntaxNodePtr, SyntaxToken,
};
pub use url_spec::UrlSpec;
pub use version_spec::{VersionClauseNode, VersionSpecNode};

/// Parse a PEP 508 requirement string into an AST
pub fn parse(source: &str) -> (SyntaxNode, Vec<crate::Error>) {
    let parser = crate::parser::Parser::new(source);
    parser.parse_ast()
}
