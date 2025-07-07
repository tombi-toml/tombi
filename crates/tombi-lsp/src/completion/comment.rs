use tombi_ast::AstToken;
use tower_lsp::lsp_types::Url;

use super::{CompletionContent, CompletionEdit};

pub fn get_comment_completion_contents(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    text_document_uri: &Url,
) -> Option<Vec<CompletionContent>> {
    let mut in_comments = false;
    if let Some(comments) = root.get_document_header_comments() {
        for comment in comments {
            let comment_range = comment.syntax().range();

            if comment_range.contains(position) {
                in_comments = true;
                let comment_text = comment.syntax().text();
                if comment_text.starts_with('#') {
                    if let Some(colon_pos) = comment_text.find(':') {
                        if comment_text[1..colon_pos]
                            .chars()
                            .all(|c| c.is_whitespace())
                        {
                            let mut directive_range = comment_range;
                            directive_range.end.column =
                                comment_range.start.column + 1 + colon_pos as u32;
                            let mut completion_contents = Vec::new();

                            if root.file_schema_url(None).is_none() {
                                completion_contents.push(CompletionContent::new_comment_directive(
                                    "schema",
                                    "Schema URL/Path",
                                    "This directive specifies the schema URL or path for the document.",
                                    CompletionEdit::new_comment_schema_directive(
                                        position,
                                        directive_range,
                                        text_document_uri,
                                    ),
                                ));
                            }

                            return Some(completion_contents);
                        }
                    }
                }
            }
        }
    }

    if in_comments {
        Some(Vec::with_capacity(0))
    } else {
        None
    }
}
