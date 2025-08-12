use tombi_ast::AstToken;
use tombi_comment_directive::TOMBI_COMMENT_DIRECTIVE_VERSION;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tower_lsp::lsp_types::Url;

use crate::completion::{extract_keys_and_hint, find_completion_contents_with_tree};

use super::{CompletionContent, CompletionEdit};

pub async fn get_comment_completion_contents(
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

                            // Add schema directive completion if not already present
                            if root.schema_directive(None).is_none() {
                                completion_contents.push(CompletionContent::new_schema_directive(
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
                        } else if comment_text[1..colon_pos].trim_start() == "tombi" {
                            let toml_version = TOMBI_COMMENT_DIRECTIVE_VERSION;
                            let tombi_directive = &comment_text[colon_pos + 1..];
                            if comment_range.start.column + (colon_pos as u32) < position.column {
                                let mut position_in_content = position;
                                position_in_content.line = 0;
                                position_in_content.column -= colon_pos as u32 + 1;

                                let mut position_in_directive = comment_range.start;
                                position_in_directive.column += colon_pos as u32 + 1;

                                let (root, _) = tombi_parser::parse(tombi_directive, toml_version)
                                    .into_root_and_errors();

                                let Some((keys, completion_hint)) =
                                    extract_keys_and_hint(&root, position_in_content, toml_version)
                                else {
                                    return Some(Vec::with_capacity(0));
                                };

                                let document_tree =
                                    root.into_document_tree_and_errors(toml_version).tree;

                                let document_schema =
                                tombi_comment_directive::root_comment_directive_document_schema()
                                    .await;
                                let schema_context = tombi_schema_store::SchemaContext {
                                    toml_version,
                                    root_schema: Some(&document_schema),
                                    sub_schema_url_map: None,
                                    store: tombi_comment_directive::schema_store().await,
                                };

                                return Some(
                                    find_completion_contents_with_tree(
                                        &document_tree,
                                        position_in_content,
                                        &keys,
                                        &schema_context,
                                        completion_hint,
                                    )
                                    .await
                                    .into_iter()
                                    .map(|content| content.with_position(position_in_directive))
                                    .collect(),
                                );
                            }
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
