use tombi_ast_syntax::AstNode;

use crate::editor::change::Change;

pub(in crate::editor) fn array_comma_trailing_comment(
    value: &tombi_ast_syntax::Value,
    comma: Option<&tombi_ast_syntax::Comma>,
    should_append_missing_comma: bool,
) -> Vec<Change> {
    match comma {
        Some(comma)
            if value.trailing_comment().is_some()
                && comma.leading_comments().next().is_none()
                && comma.trailing_comment().is_none() =>
        {
            let trailing_comment = value.trailing_comment().unwrap();
            let comma_with_trailing_comment = format!(",{}", trailing_comment.syntax().text());
            vec![
                Change::remove_token(trailing_comment.syntax()),
                Change::append_replacing(value, comma, comma_with_trailing_comment),
            ]
        }
        None if should_append_missing_comma => {
            if let Some(trailing_comment) = value.trailing_comment() {
                let comma_with_trailing_comment = format!(",{}", trailing_comment.syntax().text());
                vec![
                    Change::remove_token(trailing_comment.syntax()),
                    Change::append(value, comma_with_trailing_comment),
                ]
            } else {
                vec![Change::append(value, ",".to_owned())]
            }
        }
        _ => Vec::new(),
    }
}
