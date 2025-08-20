mod builder;
mod error;
mod event;
mod marker;
mod output;
mod parse;
mod parsed;
mod parser;
mod token_set;

pub use error::{Error, ErrorKind};
pub use event::Event;
use itertools::Itertools;
use output::Output;
use parse::Parse;
pub use parsed::Parsed;
pub use tombi_syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

pub fn parse(source: &str, toml_version: tombi_config::TomlVersion) -> Parsed<SyntaxNode> {
    parse_as::<tombi_ast::Root>(source, toml_version)
}

/// Parses the source code and returns a syntax tree of the document header comments.
///
/// This function checks for schema URL specification in the first comment of the document
/// and uses it to determine the TOML version information.
pub fn parse_document_header_comments(source: &str) -> Parsed<SyntaxNode> {
    let lexed = tombi_lexer::lex_document_header_comments(source);
    let mut p = crate::parser::Parser::new(source, None, &lexed.tokens);

    tombi_ast::Root::parse(&mut p);

    let (tokens, events) = p.finish();

    let output = crate::event::process(events);

    let (green_tree, errs) = build_green_tree(source, &tokens, output);

    let mut errors = lexed.errors.into_iter().map(Into::into).collect_vec();

    errors.extend(errs);

    Parsed::new(green_tree, errors)
}

#[allow(private_bounds)]
pub fn parse_as<P: Parse>(
    source: &str,
    toml_version: tombi_config::TomlVersion,
) -> Parsed<SyntaxNode> {
    let lexed = tombi_lexer::lex(source);
    let mut p = crate::parser::Parser::new(source, Some(toml_version), &lexed.tokens);

    P::parse(&mut p);

    let (tokens, events) = p.finish();

    let output = crate::event::process(events);

    let (green_tree, errs) = build_green_tree(source, &tokens, output);

    let mut errors = lexed.errors.into_iter().map(Into::into).collect_vec();

    errors.extend(errs);

    Parsed::new(green_tree, errors)
}

pub fn build_green_tree(
    source: &str,
    tokens: &[tombi_lexer::Token],
    parser_output: crate::Output,
) -> (tombi_rg_tree::GreenNode, Vec<crate::Error>) {
    let mut builder = tombi_syntax::SyntaxTreeBuilder::<crate::Error>::default();

    builder::intersperse_trivia(source, tokens, &parser_output, &mut |step| match step {
        builder::Step::AddToken { kind, text } => {
            builder.token(kind, text);
        }
        builder::Step::StartNode { kind } => {
            builder.start_node(kind);
        }
        builder::Step::FinishNode => builder.finish_node(),
        builder::Step::Error { error } => builder.error(error),
    });

    builder.finish()
}

#[cfg(test)]
#[macro_export]
macro_rules! test_parser {
    {#[test] fn $name:ident($source:expr) -> Ok(_)} => {
        test_parser! {
            #[test]
            fn $name($source, Default::default()) -> Ok(_)
        }
    };

    {#[test] fn $name:ident($source:expr, $toml_version:expr) -> Ok(_)} => {
        #[test]
        fn $name() {
            let p = $crate::parse(textwrap::dedent($source).trim(), $toml_version);
            pretty_assertions::assert_eq!(
                p.errors,
                Vec::<$crate::Error>::new()
            )
        }
    };

    {#[test] fn $name:ident($source:expr) -> Err(
        [
            $(
                SyntaxError(
                    $error_kind:ident,
                    $line1:literal:$column1:literal..$line2:literal:$column2:literal
                )
            ),*$(,)*
        ]
    )} => {
        $crate::test_parser! {#[test] fn $name($source, Default::default()) -> Err([$(SyntaxError($error_kind, $line1:$column1..$line2:$column2)),*])}
    };

    {#[test] fn $name:ident($source:expr, $toml_version:expr) -> Err(
        [
            $(
                SyntaxError(
                    $error_kind:ident,
                    $line1:literal:$column1:literal..$line2:literal:$column2:literal
                )
            ),*$(,)*
        ]
    )} => {
        #[test]
        fn $name() {
            let p = $crate::parse(textwrap::dedent($source).trim(), $toml_version);

            pretty_assertions::assert_eq!(
                p.errors,
                vec![$($crate::Error::new($error_kind, (($line1, $column1), ($line2, $column2)).into())),*]
            );
        }
    };
}
