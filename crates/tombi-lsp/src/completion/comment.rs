use tombi_ast::AstToken;

use super::{CompletionContent, CompletionEdit};

pub fn get_comment_completion_contents(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
) -> Vec<CompletionContent> {
    if root.file_schema_url(None).is_some() {
        return Vec::with_capacity(0);
    }

    if let Some(comments) = root.get_document_header_comments() {
        for comment in comments {
            let comment_range = comment.syntax().range();

            if comment_range.contains(position) {
                let comment_text = comment.syntax().text();

                if (position.column - comment_range.start.column == 2)
                    && comment_text.get(0..2) == Some("#:")
                {
                    return vec![CompletionContent::new_comment_directive(
                        "schema",
                        "Schema URL/Path",
                        "This directive specifies the schema URL or path for the document.",
                        CompletionEdit::new_comment_schema_directive(position),
                    )];
                }
            }
        }
    }
    Vec::with_capacity(0)
}
