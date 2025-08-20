use tombi_ast::AstToken;
use tombi_comment_directive::TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tower_lsp::lsp_types::Url;

use crate::{
    comment_directive::{
        get_tombi_document_comment_directive, DocumentTombiDirectiveContent,
        TombiDocumentCommentDirective,
    },
    completion::{extract_keys_and_hint, find_completion_contents_with_tree},
    DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION, DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
    DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION, DOCUMENT_TOMBI_DIRECTIVE_TITLE,
};

use super::{CompletionContent, CompletionEdit};

pub async fn get_comment_directive_completion_contents(
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
                            let mut prefix_range = comment_range;
                            prefix_range.end.column =
                                comment_range.start.column + 1 + colon_pos as u32;

                            if comment_text[colon_pos + 1..]
                                .chars()
                                .all(|c| c.is_whitespace())
                            {
                                return Some(document_comment_directive_completion_contents(
                                    root,
                                    position,
                                    prefix_range,
                                    text_document_uri,
                                ));
                            }

                            if let Some(completions) =
                                document_tombi_directive_completion_contents(&comment, position)
                                    .await
                            {
                                return Some(completions);
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

fn document_comment_directive_completion_contents(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    prefix_range: tombi_text::Range,
    text_document_uri: &Url,
) -> Vec<CompletionContent> {
    let mut completion_contents = Vec::new();

    let source_path = text_document_uri.to_file_path().ok();

    // Add schema directive completion if not already present
    if root
        .schema_document_comment_directive(source_path.as_deref())
        .is_none()
    {
        completion_contents.push(CompletionContent::new_comment_directive(
            "schema",
            DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
            DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION,
            CompletionEdit::new_schema_comment_directive(position, prefix_range, text_document_uri),
        ));
    }
    completion_contents.push(CompletionContent::new_comment_directive(
        "tombi",
        DOCUMENT_TOMBI_DIRECTIVE_TITLE,
        DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION,
        CompletionEdit::new_comment_directive("tombi", position, prefix_range),
    ));

    completion_contents
}

async fn document_tombi_directive_completion_contents(
    comment: &tombi_ast::Comment,
    position: tombi_text::Position,
) -> Option<Vec<CompletionContent>> {
    if let Some(TombiDocumentCommentDirective::Content(DocumentTombiDirectiveContent {
        content,
        position_in_content,
        content_range,
    })) = get_tombi_document_comment_directive(comment, position)
    {
        let toml_version = TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
        let (root, _) = tombi_parser::parse(&content, toml_version).into_root_and_errors();

        let Some((keys, completion_hint)) =
            extract_keys_and_hint(&root, position_in_content, toml_version)
        else {
            return Some(Vec::with_capacity(0));
        };

        let document_tree = root.into_document_tree_and_errors(toml_version).tree;

        let document_schema =
            tombi_comment_directive::document_comment_directive_document_schema().await;
        let schema_context = tombi_schema_store::SchemaContext {
            toml_version,
            root_schema: Some(&document_schema),
            sub_schema_uri_map: None,
            store: tombi_comment_directive::schema_store().await,
            strict: None,
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
            .map(|content| content.with_position(content_range.start))
            .collect(),
        );
    }
    None
}
