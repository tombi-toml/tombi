pub mod ast;
mod error;
mod language;
pub mod parse;
mod syntax_kind;

pub mod lexer;
pub mod parser;

pub use error::{Error, ErrorKind};
pub use language::Pep508Language;
pub use lexer::{Cursor, Lexed, Token};
pub use syntax_kind::SyntaxKind;

// Re-export from ast module
pub use ast::{
    parse,
    // AST traits
    AstNode,
    AstToken,
    ExtrasList,
    // AST tokens
    Identifier,
    MarkerExpr,
    MarkerExpression,
    PackageName,
    ParseError,
    Pep508Requirement,
    PreorderWithTokens,
    Requirement,
    // AST nodes
    Root,
    SyntaxElement,
    SyntaxElementChildren,
    SyntaxNode,
    SyntaxNodeChildren,
    SyntaxNodePtr,
    SyntaxToken,
    SyntaxTreeBuilder,
    UrlSpec,
    VersionClause,
    VersionClauseNode,
    VersionOperator,
    VersionSpec,
    VersionSpecNode,
    VersionString,
};

// Re-export from parser module
pub use parser::{Parser, PartialParseResult};
