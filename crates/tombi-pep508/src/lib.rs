pub mod ast;
mod cursor;
mod error;
mod language;
mod lexed;
pub mod parse;
mod syntax_kind;
mod token;

pub mod lexer;
pub mod parser;

pub use error::{Error, ErrorKind};
pub use language::Pep508Language;
pub use lexed::Lexed;
pub use syntax_kind::SyntaxKind;
pub use token::Token;

// Re-export from ast module
pub use ast::{
    parse, MarkerExpression, ParseError, Pep508Requirement, PreorderWithTokens, SyntaxElement,
    SyntaxElementChildren, SyntaxNode, SyntaxNodeChildren, SyntaxNodePtr, SyntaxToken,
    SyntaxTreeBuilder, VersionClause, VersionOperator, VersionSpec,
    // AST traits
    AstNode, AstToken,
    // AST nodes
    Root, Requirement, PackageName, ExtrasList, VersionSpecNode, VersionClauseNode, UrlSpec, MarkerExpr,
    // AST tokens
    Identifier, VersionString,
};

// Re-export from parser module
pub use parser::{Parser, PartialParseResult};