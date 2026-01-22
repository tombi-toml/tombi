use itertools::Itertools;
use tombi_ast::{AstToken, SchemaDocumentCommentDirective};
use tombi_comment_directive::{
    TOMBI_COMMENT_DIRECTIVE_TOML_VERSION, TombiCommentDirectiveImpl,
    document::TombiDocumentDirectiveContent,
};
use tombi_comment_directive_store::comment_directive_document_schema;
use tombi_document_tree::IntoDocumentTreeAndErrors;
use tombi_extension::{CompletionContentPriority, CompletionKind};
use tombi_uri::{SchemaUri, Uri};

use crate::{
    DOCUMENT_SCHEMA_DIRECTIVE_DESCRIPTION, DOCUMENT_SCHEMA_DIRECTIVE_TITLE,
    DOCUMENT_TOMBI_DIRECTIVE_DESCRIPTION, DOCUMENT_TOMBI_DIRECTIVE_TITLE,
    comment_directive::{CommentDirectiveContext, GetCommentDirectiveContext},
    completion::{extract_keys_and_hint, find_completion_contents_with_tree},
};

use super::{CompletionContent, CompletionEdit};

use std::path::Path;

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
                let completions = get_schema_path_completions(base_dir, schema_text, schema_range);
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
    let (root, _) = tombi_parser::parse(&content, toml_version).into_root_and_errors();

    let Some((keys, completion_hint)) =
        extract_keys_and_hint(&root, position_in_content, toml_version, None)
    else {
        return Some(Vec::with_capacity(0));
    };

    let document_tree = root.into_document_tree_and_errors(toml_version).tree;

    let schema_store = tombi_comment_directive_store::schema_store().await;
    let document_schema = comment_directive_document_schema(schema_store, schema_uri).await;
    let source_schema = tombi_schema_store::SourceSchema {
        root_schema: Some(document_schema),
        sub_schema_uri_map: ahash::AHashMap::with_capacity(0),
    };
    let schema_context = tombi_schema_store::SchemaContext {
        toml_version,
        root_schema: source_schema.root_schema.as_ref(),
        sub_schema_uri_map: None,
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

/// Get file path completions for schema directive
fn get_schema_path_completions(
    base_dir: &Path,
    schema_text: &str,
    schema_range: tombi_text::Range,
) -> Vec<CompletionContent> {
    let mut completions = Vec::new();

    // Parse current path to get directory, prefix, and path prefix for completion
    let (search_dir, prefix, path_prefix) = if schema_text.is_empty() {
        (base_dir.to_path_buf(), String::new(), String::new())
    } else {
        let path = Path::new(schema_text);
        if schema_text.ends_with('/') {
            // User is at a directory, show contents of that directory
            // e.g., "docs/" -> search in "docs/", prefix is empty, path_prefix is "docs/"
            (base_dir.join(path), String::new(), schema_text.to_string())
        } else {
            // User is typing a name, show matching items in parent directory
            // e.g., "docs/sch" -> search in "docs/", prefix is "sch", path_prefix is "docs/"
            let parent = path.parent().unwrap_or(Path::new(""));
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let prefix_str = if parent == Path::new("") {
                String::new()
            } else {
                format!("{}/", parent.display())
            };
            (base_dir.join(parent), file_name.to_string(), prefix_str)
        }
    };

    // Read directory contents
    let Ok(entries) = std::fs::read_dir(&search_dir) else {
        return completions;
    };

    for entry in entries.flatten() {
        let Ok(file_name) = entry.file_name().into_string() else {
            continue;
        };

        // Skip hidden files
        if file_name.starts_with('.') {
            continue;
        }

        // Filter by prefix
        if !prefix.is_empty() && !file_name.starts_with(&prefix) {
            continue;
        }

        let Ok(metadata) = entry.metadata() else {
            continue;
        };

        let is_dir = metadata.is_dir();
        let is_json = file_name.ends_with(".json");

        // Only include directories and .json files
        if !is_dir && !is_json {
            continue;
        }

        // Calculate the relative path by combining path_prefix and file_name
        let relative_path = match (path_prefix.is_empty(), is_dir) {
            (true, true) => format!("{}/", file_name),
            (true, false) => file_name.clone(),
            (false, true) => format!("{}{}/", path_prefix, file_name),
            (false, false) => format!("{}{}", path_prefix, file_name),
        };

        // Use the full relative path as the label
        let label = relative_path.clone();

        let detail = if is_dir {
            "Directory".to_string()
        } else {
            "JSON file".to_string()
        };

        let completion_edit =
            CompletionEdit::new_string_literal_while_editing(&relative_path, schema_range);

        if let Some(edit) = completion_edit {
            completions.push(CompletionContent {
                label,
                kind: CompletionKind::String,
                emoji_icon: if is_dir { Some('📁') } else { Some('📄') },
                priority: CompletionContentPriority::Custom("40".to_string()),
                detail: Some(detail),
                documentation: None,
                filter_text: None,
                schema_uri: None,
                deprecated: None,
                edit: Some(edit),
                preselect: None,
                in_comment: false,
            });
        }
    }

    completions
}
