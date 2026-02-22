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

pub fn parse(source: &str) -> Parsed<SyntaxNode> {
    parse_as::<tombi_ast::Root>(source)
}

#[allow(private_bounds)]
pub fn parse_as<P: Parse>(source: &str) -> Parsed<SyntaxNode> {
    let lexed = tombi_lexer::lex(source);
    let mut p = crate::parser::Parser::new(source, &lexed.tokens);

    P::parse(&mut p);

    let (tokens, events) = p.finish();

    let output = crate::event::process(events);

    let (green_tree, errs) = build_green_tree(source, &tokens, output);

    let mut errors = lexed.errors.into_iter().map(Into::into).collect_vec();

    errors.extend(errs);

    Parsed::new(green_tree, errors, lexed.line_ending)
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
        #[test]
        fn $name() {
            tombi_test_lib::init_log();

            let p = $crate::parse(textwrap::dedent($source).trim());

            log::debug!("syntax_node: {:#?}", p.syntax_node());

            pretty_assertions::assert_eq!(
                p.errors,
                Vec::<$crate::Error>::new()
            );
        }
    };

    {#[test] fn $name:ident($source:expr) -> Ok(|$root:ident| -> $assert_expr:expr)} => {
        #[test]
        fn $name() {
            tombi_test_lib::init_log();

            let p = $crate::parse(textwrap::dedent($source).trim());

            log::debug!("syntax_node: {:#?}", p.syntax_node());

            pretty_assertions::assert_eq!(
                p.errors,
                Vec::<$crate::Error>::new()
            );

            use tombi_ast::AstNode as _;
            let $root = tombi_ast::Root::cast(p.syntax_node())
                .expect("parse result must contain ROOT syntax node");

            assert!(
                $assert_expr,
                "Ok(|root| -> ...) assertion failed: {}",
                stringify!($assert_expr)
            );
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
        #[test]
        fn $name() {
            tombi_test_lib::init_log();

            let p = $crate::parse(textwrap::dedent($source).trim());

            log::debug!("syntax_node: {:#?}", p.syntax_node());

            pretty_assertions::assert_eq!(
                p.errors,
                vec![$($crate::Error::new($error_kind, (($line1, $column1), ($line2, $column2)).into())),*]
            );
        }
    };

}
