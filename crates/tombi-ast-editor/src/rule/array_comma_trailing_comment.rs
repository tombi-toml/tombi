use tombi_ast::AstNode;
use tombi_schema_store::SchemaContext;
use tombi_syntax::SyntaxElement;

use crate::{change::Change, node::make_comma_with_trailing_comment};

pub fn array_comma_trailing_comment(
    value: &tombi_ast::Value,
    comma: Option<&tombi_ast::Comma>,
    _schema_context: &SchemaContext,
) -> Vec<Change> {
    if let Some(trailing_comment) = value.trailing_comment() {
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
                    base: SyntaxElement::Node(value.syntax().clone()),
                    new: vec![SyntaxElement::Node(comma_with_trailing_comment)],
                },
            ];
        }
    }

    Vec::with_capacity(0)
}
