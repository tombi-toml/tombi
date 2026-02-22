mod builder;
mod error;
mod event;
mod marker;
mod output;
mod parse;
mod parsed;
mod parser;
mod support;
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

pub fn format_tree_as_macro(node: &SyntaxNode, base_indent: usize) -> String {
    use tombi_rg_tree::NodeOrToken;
    fn fmt_items(
        node: &SyntaxNode,
        indent: usize,
        out: &mut String,
    ) {
        let children: Vec<_> = node.children_with_tokens().collect();
        for (i, child) in children.iter().enumerate() {
            let prefix = "    ".repeat(indent);
            let comma = if i < children.len() - 1 { "," } else { "" };
            match child {
                NodeOrToken::Token(t) => {
                    let kind = format!("{:?}", t.kind());
                    let value = t.text().to_string();
                    out.push_str(&format!("{}{}: {:?}{}\n", prefix, kind, value, comma));
                }
                NodeOrToken::Node(n) => {
                    let kind = format!("{:?}", n.kind());
                    out.push_str(&format!("{}{}: {{\n", prefix, kind));
                    fmt_items(n, indent + 1, out);
                    out.push_str(&format!("{}}}{}\n", prefix, comma));
                }
            }
        }
    }
    let mut out = String::new();
    fmt_items(node, base_indent, &mut out);
    out
}

#[cfg(test)]
#[derive(PartialEq, Eq)]
pub enum SyntaxTreePattern {
    Token(String, String),
    Node(String, Vec<SyntaxTreePattern>),
}

#[cfg(test)]
pub fn syntax_node_to_patterns(node: &SyntaxNode) -> Vec<SyntaxTreePattern> {
    use tombi_rg_tree::NodeOrToken;
    node.children_with_tokens()
        .map(|child| match child {
            NodeOrToken::Token(t) => SyntaxTreePattern::Token(
                format!("{:?}", t.kind()),
                t.text().to_string(),
            ),
            NodeOrToken::Node(n) => SyntaxTreePattern::Node(
                format!("{:?}", n.kind()),
                syntax_node_to_patterns(&n),
            ),
        })
        .collect()
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
        let mut __items: Vec<$crate::SyntaxTreePattern> = Vec::new();
        $crate::syntax_tree_items!(__items; $($tt)*);
        __items
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

            log::debug!("syntax_node: {:#?}", p.syntax_node());

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

            log::debug!("syntax_node: {:#?}", p.syntax_node());

            pretty_assertions::assert_eq!(
                p.errors,
                Vec::<$crate::Error>::new()
            );

            let expected = $crate::syntax_tree!($($expected)*);
            let actual = $crate::syntax_node_to_patterns(&p.syntax_node());
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
