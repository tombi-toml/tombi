use tombi_ast::AstNode;
use tombi_syntax::SyntaxElement;

use crate::{change::Change, node::make_comma_with_trailing_comment};

pub fn inline_table_comma_trailing_comment(
    key_value: &tombi_ast::KeyValue,
    comma: Option<&tombi_ast::Comma>,
) -> Vec<Change> {
    if let Some(trailing_comment) = key_value.trailing_comment() {
        if match comma {
            Some(comma) => {
                comma.trailing_comment().is_none() && comma.leading_comments().next().is_none()
            }
            None => true,
        } {
            let comma_with_trailing_comment = make_comma_with_trailing_comment(&trailing_comment);

            return vec![
                Change::Remove {
                    target: SyntaxElement::Token(trailing_comment.syntax().clone()),
                },
                Change::Append {
                    base: SyntaxElement::Node(key_value.syntax().clone()),
                    new: vec![SyntaxElement::Node(comma_with_trailing_comment)],
                },
            ];
        }
    }

    Vec::with_capacity(0)
}
