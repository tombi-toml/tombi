use tombi_ast_syntax::AstNode;

use crate::editor::change::Change;

pub(in crate::editor) fn inline_table_comma_trailing_comment<'t>(
    key_value: &tombi_ast_syntax::KeyValue<'t>,
    comma: Option<&tombi_ast_syntax::Comma<'t>>,
    should_append_missing_comma: bool,
) -> Vec<Change> {
    match comma {
        Some(comma)
            if key_value.trailing_comment().is_some()
                && comma.trailing_comment().is_none()
                && comma.leading_comments().next().is_none() =>
        {
            let trailing_comment = key_value.trailing_comment().unwrap();
            vec![
                Change::remove_token(trailing_comment.syntax()),
                Change::append_comma_with_comment_replacing(
                    key_value,
                    comma,
                    trailing_comment.syntax(),
                ),
            ]
        }
        None if should_append_missing_comma => {
            if let Some(trailing_comment) = key_value.trailing_comment() {
                vec![
                    Change::remove_token(trailing_comment.syntax()),
                    Change::append_comma_with_comment(key_value, trailing_comment.syntax()),
                ]
            } else {
                vec![Change::append(key_value, ",".to_owned())]
            }
        }
        _ => Vec::new(),
    }
}
