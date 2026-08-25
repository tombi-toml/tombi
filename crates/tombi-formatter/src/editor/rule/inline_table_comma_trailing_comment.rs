use tombi_ast_syntax::AstNode;

use crate::editor::change::Change;

pub(in crate::editor) fn inline_table_comma_trailing_comment(
    key_value: &tombi_ast_syntax::KeyValue,
    comma: Option<&tombi_ast_syntax::Comma>,
    should_append_missing_comma: bool,
) -> Vec<Change> {
    match comma {
        Some(comma)
            if key_value.trailing_comment().is_some()
                && comma.trailing_comment().is_none()
                && comma.leading_comments().next().is_none() =>
        {
            let trailing_comment = key_value.trailing_comment().unwrap();
            let comma_with_trailing_comment = format!(",{}", trailing_comment.syntax().text());
            vec![
                Change::remove_token(trailing_comment.syntax()),
                Change::append_replacing(key_value, comma, comma_with_trailing_comment),
            ]
        }
        None if should_append_missing_comma => {
            if let Some(trailing_comment) = key_value.trailing_comment() {
                let comma_with_trailing_comment = format!(",{}", trailing_comment.syntax().text());
                vec![
                    Change::remove_token(trailing_comment.syntax()),
                    Change::append(key_value, comma_with_trailing_comment),
                ]
            } else {
                vec![Change::append(key_value, ",".to_owned())]
            }
        }
        _ => Vec::new(),
    }
}
