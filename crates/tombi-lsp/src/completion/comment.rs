use tombi_ast_syntax::{AstNode, AstToken, SchemaDocumentCommentDirective};
use tombi_comment_directive::{
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION, TombiCommentDirectiveImpl,
    document::TombiDocumentDirectiveContent,
};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_document_tree_syntax::IntoDocumentTreeAndErrors;
use tombi_extension::get_file_path_completions;
use tombi_uri::{SchemaUri, Uri};

use crate::{
    DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION, DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
    DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION, DOCUMENT_TOMBI_DIRECTIVE_TITLE,
    comment_directive::{CommentDirectiveContext, GetCommentDirectiveContext},
    completion::{extract_keys_and_hint, find_completion_contents},
};

use super::{CompletionContent, CompletionEdit};

pub async fn get_document_comment_directive_completion_contents(
    root: &tombi_ast_syntax::Root<'_>,
    comment: &tombi_ast_syntax::Comment<'_>,
    offset: tombi_text::Offset,
    text_document_uri: &Uri,
) -> Option<Vec<CompletionContent>> {
    let comment_text = comment.syntax().text();
    if let Some(colon_pos) = comment_text.find(':')
        && comment_text[1..colon_pos]
            .chars()
            .all(|c| c.is_whitespace())
    {
        let comment_span = comment.syntax().span();
        let directive_len = comment_text[colon_pos + 1..]
            .chars()
            .take_while(|c| !c.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        let directive_span = tombi_text::Span::new(
            comment_span.start,
            comment_span.start + (1 + colon_pos + directive_len) as u32,
        );

        if directive_span.contains_inclusive(offset) {
            return Some(document_comment_directive_completion_contents(
                root,
                offset,
                comment_span,
                text_document_uri,
            ));
        }

        // Check if this is a schema directive and provide file path completion
        if let Some(source_path) = text_document_uri.to_file_path().ok().as_deref()
            && let Some(schema_directive) = comment.get_document_schema_directive(Some(source_path))
            && let Some((schema_text, schema_span)) =
                get_schema_text_and_span(comment, &schema_directive)
        {
            // Check if offset is in the schema value part
            if schema_span.contains_inclusive(offset)
                && let Some(base_dir) = source_path.parent()
            {
                let completions =
                    get_file_path_completions(base_dir, schema_text, schema_span, Some(&["json"]));
                if !completions.is_empty() {
                    return Some(completions);
                }
            }
        }

        if let Some(comment_directive_context) = comment
            .get_tombi_document_directive()
            .and_then(|directive| directive.get_context(offset))
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
    root: &tombi_ast_syntax::Root<'_>,
    offset: tombi_text::Offset,
    comment_span: tombi_text::Span,
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
            CompletionEdit::new_schema_comment_directive(offset, comment_span, text_document_uri),
        ));
    }
    completion_contents.push(CompletionContent::new_comment_directive(
        "tombi",
        DOCUMENT_TOMBI_DIRECTIVE_TITLE,
        DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION,
        CompletionEdit::new_comment_directive("tombi", offset, comment_span),
    ));

    completion_contents
}

pub async fn get_tombi_comment_directive_content_completion_contents(
    comment_directive_context: CommentDirectiveContext<String>,
    schema_uri: SchemaUri,
) -> Option<Vec<CompletionContent>> {
    let CommentDirectiveContext::Content {
        content,
        content_span,
        offset_in_content,
    } = comment_directive_context
    else {
        return None;
    };

    let toml_version = TOMBI_COMMENT_DIRECTIVE_TOML_VERSION;
    let parsed = tombi_parser::parse(&content);
    let root = parsed.root();
    let decoded = root.decode_strings(toml_version);

    let Some((keys, completion_hint)) =
        extract_keys_and_hint(&root, &decoded, offset_in_content, toml_version, None)
    else {
        return Some(Vec::new());
    };

    // The positions are in the content of the directive, which is parsed as a separate document.
    let line_index = parsed.line_index();
    let document_tree = root
        .into_document_tree_and_errors(toml_version, &decoded)
        .tree;

    let schema_store = tombi_comment_directive_store::schema_store().await;
    let document_schema = comment_directive_document_schema(schema_store, schema_uri).await;
    let source_schema = tombi_schema_store::SourceSchema::new(
        Some(document_schema),
        tombi_hashmap::IndexMap::with_capacity(0),
        Some(toml_version),
        None,
        Default::default(),
        Default::default(),
        Default::default(),
    );
    let schema_context = tombi_schema_store::SchemaContext::from_source_schema(
        toml_version,
        Some(&source_schema),
        schema_store,
        None,
    );

    Some(
        find_completion_contents(
            &document_tree,
            crate::CursorPosition::new(offset_in_content, line_index),
            &keys,
            &schema_context,
            completion_hint,
        )
        .await
        .into_iter()
        .map(|mut content| {
            content.in_comment = true;
            content.with_offset(content_span.start)
        })
        .collect(),
    )
}

fn get_schema_text_and_span<'a>(
    comment: &'a tombi_ast_syntax::Comment<'_>,
    schema_directive: &'a SchemaDocumentCommentDirective,
) -> Option<(&'a str, tombi_text::Span)> {
    let schema_text = match &schema_directive.uri {
        Ok(schema_uri) if matches!(schema_uri.scheme(), "file") => {
            let comment_start = comment.syntax().span().start;
            &comment.syntax().text()[schema_directive.uri_span - comment_start]
        }
        Err(schema_path) => schema_path,
        Ok(_) => return None,
    };

    Some((schema_text, schema_directive.uri_span))
}
