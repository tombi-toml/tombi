mod builder;
mod error;
mod event;
mod marker;
mod parse;
mod parsed;
mod parser;
mod support;
mod token_set;

pub use error::{Error, ErrorKind};
use itertools::Itertools;
pub use parsed::ParseResult;
use tombi_ast_syntax::{SyntaxKind, SyntaxNode};

pub fn parse(source: &str) -> ParseResult {
    let (syntax, errors, line_ending) = parse_syntax::<tombi_ast_syntax::Root>(source);
    ParseResult::new(syntax, errors, line_ending)
}

fn parse_syntax<P: parse::Parse>(
    source: &str,
) -> (SyntaxNode, Vec<crate::Error>, tombi_text::LineEnding) {
    let lexed = tombi_lexer::lex(source);
    let mut p = crate::parser::Parser::new(source, &lexed.tokens);

    P::parse(&mut p);

    let (events, synthetic_tokens, errs) = p.finish();

    let syntax = build_syntax_tape(source, &lexed.tokens, &synthetic_tokens, &events);

    let mut errors = lexed.errors.into_iter().map(Into::into).collect_vec();

    errors.extend(errs);

    (syntax, errors, lexed.line_ending)
}

fn build_syntax_tape(
    source: &str,
    tokens: &[tombi_lexer::Token],
    synthetic_tokens: &[tombi_lexer::Token],
    events: &[crate::event::Event],
) -> SyntaxNode {
    let mut builder = tombi_ast_syntax::SyntaxTreeBuilder::with_capacity(source, events.len());
    let mut offset = tombi_text::Offset::default();

    builder::intersperse_trivia(tokens, synthetic_tokens, events, |step| match step {
        builder::Step::AddToken { kind, span } => {
            builder.token(kind, span);
            offset = span.end;
        }
        builder::Step::StartNode { kind } => {
            builder.start_node(kind, offset);
        }
        builder::Step::FinishNode => builder.finish_node(offset),
    });

    builder.finish()
}

#[cfg(test)]
#[derive(PartialEq, Eq)]
enum TreePattern {
    Token(String, String),
    Node(String, Vec<TreePattern>),
}

#[cfg(test)]
fn tree_patterns(node: &SyntaxNode) -> Vec<TreePattern> {
    fn convert(tree: tombi_ast_syntax::DebugTree) -> TreePattern {
        match tree {
            tombi_ast_syntax::DebugTree::Node { kind, children } => TreePattern::Node(
                format!("{kind:?}"),
                children.into_iter().map(convert).collect(),
            ),
            tombi_ast_syntax::DebugTree::Token { kind, text } => {
                TreePattern::Token(format!("{kind:?}"), text)
            }
        }
    }

    match node.debug_tree() {
        tombi_ast_syntax::DebugTree::Node { children, .. } => {
            children.into_iter().map(convert).collect()
        }
        tombi_ast_syntax::DebugTree::Token { .. } => unreachable!("the syntax root is a node"),
    }
}

#[cfg(test)]
#[derive(PartialEq, Eq)]
pub enum SyntaxTreePattern {
    Token(String, String),
    Node(String, Vec<SyntaxTreePattern>),
}

#[cfg(test)]
pub fn syntax_node_to_patterns(node: &SyntaxNode) -> Vec<SyntaxTreePattern> {
    fn convert(pattern: TreePattern) -> SyntaxTreePattern {
        match pattern {
            TreePattern::Token(kind, text) => SyntaxTreePattern::Token(kind, text),
            TreePattern::Node(kind, children) => {
                SyntaxTreePattern::Node(kind, children.into_iter().map(convert).collect())
            }
        }
    }
    tree_patterns(node).into_iter().map(convert).collect()
}

#[cfg(test)]
pub fn format_tree(patterns: &[SyntaxTreePattern], indent: usize) -> String {
    let mut out = String::new();
    for p in patterns {
        let prefix = "  ".repeat(indent);
        match p {
            SyntaxTreePattern::Token(kind, value) => {
                out += &format!("{}{}: {:?}\n", prefix, kind, value);
            }
            SyntaxTreePattern::Node(kind, children) => {
                out += &format!("{}{}: {{\n", prefix, kind);
                out += &format_tree(children, indent + 1);
                out += &format!("{}}}\n", prefix);
            }
        }
    }
    out
}

#[cfg(test)]
#[macro_export]
macro_rules! syntax_tree {
    ($($tt:tt)*) => {{
        #[allow(unused_mut)]
        #[allow(clippy::vec_init_then_push)]
        {
            let mut __items: Vec<$crate::SyntaxTreePattern> = Vec::new();
            $crate::syntax_tree_items!(__items; $($tt)*);
            __items
        }
    }};
}

#[cfg(test)]
#[macro_export]
macro_rules! syntax_tree_items {
    ($items:ident;) => {};

    ($items:ident; $kind:ident : { $($inner:tt)* } , $($rest:tt)*) => {
        $items.push($crate::SyntaxTreePattern::Node(
            stringify!($kind).to_string(),
            $crate::syntax_tree!($($inner)*),
        ));
        $crate::syntax_tree_items!($items; $($rest)*);
    };

    ($items:ident; $kind:ident : { $($inner:tt)* }) => {
        $items.push($crate::SyntaxTreePattern::Node(
            stringify!($kind).to_string(),
            $crate::syntax_tree!($($inner)*),
        ));
    };

    ($items:ident; $kind:ident : $value:literal , $($rest:tt)*) => {
        $items.push($crate::SyntaxTreePattern::Token(
            stringify!($kind).to_string(),
            $value.to_string(),
        ));
        $crate::syntax_tree_items!($items; $($rest)*);
    };

    ($items:ident; $kind:ident : $value:literal) => {
        $items.push($crate::SyntaxTreePattern::Token(
            stringify!($kind).to_string(),
            $value.to_string(),
        ));
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! test_parser {
    {#[test] fn $name:ident($source:expr) -> Ok(_)} => {
        #[test]
        fn $name() {
            tombi_test_lib::init_log();

            let p = $crate::parse(textwrap::dedent($source).trim());

            log::debug!("root: {:#?}", p.root());

            pretty_assertions::assert_eq!(
                p.errors,
                Vec::<$crate::Error>::new()
            );
        }
    };

    {#[test] fn $name:ident($source:expr) -> Ok({ $($expected:tt)* })} => {
        #[test]
        fn $name() {
            tombi_test_lib::init_log();

            let p = $crate::parse(textwrap::dedent($source).trim());

            let root = p.root();
            log::debug!("root: {root:#?}");

            pretty_assertions::assert_eq!(
                p.errors,
                Vec::<$crate::Error>::new()
            );

            use tombi_ast_syntax::AstNode as _;
            let expected = $crate::syntax_tree!($($expected)*);
            let actual = $crate::syntax_node_to_patterns(root.syntax());
            pretty_assertions::assert_eq!(
                $crate::format_tree(&actual, 0),
                $crate::format_tree(&expected, 0),
            );
        }
    };

    {#[test] fn $name:ident($source:expr) -> Ok(|$root:ident| -> $assert_expr:expr)} => {
        #[test]
        fn $name() {
            tombi_test_lib::init_log();

            let p = $crate::parse(textwrap::dedent($source).trim());

            log::debug!("root: {:#?}", p.root());

            pretty_assertions::assert_eq!(
                p.errors,
                Vec::<$crate::Error>::new()
            );

            let $root = p.root();

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

            log::debug!("root: {:#?}", p.root());

            pretty_assertions::assert_eq!(
                p.errors,
                vec![$($crate::Error::new($error_kind, (($line1, $column1), ($line2, $column2)).into())),*]
            );
        }
    };

    {#[test] fn $name:ident($source:expr) -> Assert(|$parsed:ident| $assertion:block)} => {
        #[test]
        fn $name() {
            tombi_test_lib::init_log();

            let $parsed = $crate::parse(textwrap::dedent(&$source).trim());

            assert!($assertion);
        }
    };

    {#[test] fn $name:ident($source:expr) -> RawAssert(|$parsed:ident| $assertion:block)} => {
        #[test]
        fn $name() {
            tombi_test_lib::init_log();

            let $parsed = $crate::parse($source);

            assert!($assertion);
        }
    };

}
