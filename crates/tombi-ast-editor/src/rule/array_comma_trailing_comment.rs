use tombi_ast::AstNode;
use tombi_syntax::SyntaxElement;

use crate::{
    change::Change,
    node::{make_comma, make_comma_with_trailing_comment},
};

pub fn array_comma_trailing_comment(
    value: &tombi_ast::Value,
    comma: Option<&tombi_ast::Comma>,
    should_append_missing_comma: bool,
) -> Vec<Change> {
    match comma {
        Some(comma)
            if value.trailing_comment().is_some()
                && comma.leading_comments().next().is_none()
                && comma.trailing_comment().is_none() =>
        {
            let trailing_comment = value.trailing_comment().unwrap();
            let comma_with_trailing_comment = make_comma_with_trailing_comment(&trailing_comment);
            vec![
                Change::Remove {
                    target: SyntaxElement::Token(trailing_comment.syntax().clone()),
                },
                Change::Append {
                    base: SyntaxElement::Node(value.syntax().clone()),
                    new: vec![SyntaxElement::Node(comma_with_trailing_comment)],
                },
            ]
        }
        None if should_append_missing_comma => {
            if let Some(trailing_comment) = value.trailing_comment() {
                let comma_with_trailing_comment =
                    make_comma_with_trailing_comment(&trailing_comment);
                vec![
                    Change::Remove {
                        target: SyntaxElement::Token(trailing_comment.syntax().clone()),
                    },
                    Change::Append {
                        base: SyntaxElement::Node(value.syntax().clone()),
                        new: vec![SyntaxElement::Node(comma_with_trailing_comment)],
                    },
                ]
            } else {
                vec![Change::Append {
                    base: SyntaxElement::Node(value.syntax().clone()),
                    new: vec![SyntaxElement::Node(make_comma())],
                }]
            }
        }
        _ => Vec::with_capacity(0),
    }
}
