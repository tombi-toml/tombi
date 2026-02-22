use std::sync::Arc;

use itertools::Itertools;
use tombi_ast::{AstToken, SchemaDocumentCommentDirective};
use tombi_comment_directive::{
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION, TombiCommentDirectiveImpl,
    document::TombiDocumentDirectiveContent,
};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_extension::get_file_path_completions;
use tombi_uri::{SchemaUri, Uri};

use crate::{
    DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION, DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
    DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION, DOCUMENT_TOMBI_DIRECTIVE_TITLE,
    comment_directive::{CommentDirectiveContext, GetCommentDirectiveContext},
    completion::{extract_keys_and_hint, find_completion_contents_with_tree},
};

use super::{CompletionContent, CompletionEdit};

pub async fn get_document_comment_directive_completion_contents(
    root: &tombi_ast::Root,
    comment: &tombi_ast::Comment,
    position: tombi_text::Position,
    text_document_uri: &Uri,
) -> Option<Vec<CompletionContent>> {
    let comment_text = comment.syntax().text();
    if let Some(colon_pos) = comment_text.find(':')
        && comment_text[1..colon_pos]
            .chars()
            .all(|c| c.is_whitespace())
    {
        let comment_range = comment.syntax().range();
        let mut prefix_range = comment_range;
        prefix_range.end.column = comment_range.start.column + 1 + colon_pos as u32;

        let directive_len = comment_text[colon_pos + 1..]
            .chars()
            .take_while(|c| !c.is_whitespace())
            .collect_vec()
            .len();
        let mut directive_range = prefix_range;
        directive_range.end.column += directive_len as u32;

        if directive_range.contains(position) {
            return Some(document_comment_directive_completion_contents(
                root,
                position,
                comment_range,
                text_document_uri,
            ));
        }

        // Check if this is a schema directive and provide file path completion
        if let Some(source_path) = text_document_uri.to_file_path().ok().as_deref()
            && let Some(schema_directive) = comment.get_document_schema_directive(Some(source_path))
            && let Some((schema_text, schema_range)) =
                get_schema_text_and_range(comment_text, &schema_directive)
        {
            // Check if position is in the schema value part
            if schema_range.contains(position)
                && let Some(base_dir) = source_path.parent()
            {
                let completions =
                    get_file_path_completions(base_dir, schema_text, schema_range, Some(&["json"]));
                if !completions.is_empty() {
                    return Some(completions);
                }
            }
        }

        if let Some(comment_directive_context) = comment
            .get_tombi_document_directive()
            .and_then(|directive| directive.get_context(position))
            && let Some(completions) = get_tombi_comment_directive_content_completion_contents(
                comment_directive_context,
                TombiDocumentDirectiveContent::comment_directive_schema_url(),
            )
            .await
        {
            return Some(completions);
        }
    }

    None
}

fn document_comment_directive_completion_contents(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    comment_range: tombi_text::Range,
    text_document_uri: &Uri,
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
            CompletionEdit::new_schema_comment_directive(
                position,
                comment_range,
                text_document_uri,
            ),
        ));
    }
    completion_contents.push(CompletionContent::new_comment_directive(
        "tombi",
        DOCUMENT_TOMBI_DIRECTIVE_TITLE,
        DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION,
        CompletionEdit::new_comment_directive("tombi", position, comment_range),
    ));

    completion_contents
}

pub async fn get_tombi_comment_directive_content_completion_contents(
    comment_directive_context: CommentDirectiveContext<String>,
    schema_uri: SchemaUri,
) -> Option<Vec<CompletionContent>> {
    let CommentDirectiveContext::Content {
        content,
        content_range,
        position_in_content,
    } = comment_directive_context
    else {
        return None;
    };

    let toml_version = TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
    let (root, _) = tombi_parser::parse(&content).into_root_and_errors();

    let Some((keys, completion_hint)) =
        extract_keys_and_hint(&root, position_in_content, toml_version, None)
    else {
        return Some(Vec::with_capacity(0));
    };

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;

    let schema_store = tombi_comment_directive_store::schema_store().await;
    let document_schema = comment_directive_document_schema(schema_store, schema_uri).await;
    let source_schema = tombi_schema_store::SourceSchema {
        root_schema: Some(Arc::new(document_schema)),
        sub_schema_uri_map: ahash::AHashMap::with_capacity(0),
    };
    let schema_context = tombi_schema_store::SchemaContext {
        toml_version,
        root_schema: source_schema.root_schema.as_deref(),
        sub_schema_uri_map: None,
        schema_visits: Default::default(),
        store: schema_store,
        strict: None,
    };

    Some(
        find_completion_contents_with_tree(
            &document_tree,
            position_in_content,
            &keys,
            &schema_context,
            completion_hint,
        )
        .await
        .into_iter()
        .map(|mut content| {
            content.in_comment = true;
            content.with_position(content_range.start)
        })
        .collect(),
    )
}

fn get_schema_text_and_range<'a>(
    comment_text: &'a str,
    schema_directive: &'a SchemaDocumentCommentDirective,
) -> Option<(&'a str, tombi_text::Range)> {
    let schema_text = match &schema_directive.uri {
        Ok(schema_uri) if matches!(schema_uri.scheme(), "file") => {
            &comment_text[schema_directive.uri_range.start.column as usize
                ..schema_directive.uri_range.end.column as usize]
        }
        Err(schema_path) => schema_path,
        Ok(_) => return None,
    };

    Some((schema_text, schema_directive.uri_range))
}
