use tombi_ast::{algo::ancestors_at_position, AstNode};
use tombi_schema_store::{Accessor, AccessorContext};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier,
    TextDocumentEdit, TextEdit, WorkspaceEdit,
};

pub enum CodeActionRefactorRewriteName {
    DottedKeysToInlineTable,
    InlineTableToDottedKeys,
}

impl std::fmt::Display for CodeActionRefactorRewriteName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeActionRefactorRewriteName::DottedKeysToInlineTable => {
                write!(f, "Convert Dotted Keys to Inline Table")
            }
            CodeActionRefactorRewriteName::InlineTableToDottedKeys => {
                write!(f, "Convert Inline Table to Dotted Keys")
            }
        }
    }
}

pub fn dot_keys_to_inline_table_code_action(
    text_document_uri: &tombi_uri::Uri,
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    _accessors: &[Accessor],
    _contexts: &[AccessorContext],
) -> Option<CodeAction> {
    // Find the KeyValue node at the position
    let key_value = find_key_value_at_position(root, position)?;

    // Check if this is a dotted key structure
    let keys = key_value.keys()?;
    if keys.keys().count() < 2 {
        return None;
    }

    // Get the parent key (first key) and the child keys (remaining keys)
    let parent_key = keys.keys().next()?;
    let child_keys: Vec<_> = keys.keys().skip(1).collect();

    // Get the value
    let value = key_value.value()?;

    // Check if the value is a simple value (not an inline table)
    if matches!(value, tombi_ast::Value::InlineTable(_)) {
        return None;
    }

    // Build the child key string (e.g., "bar.baz" for nested keys)
    let child_key_text = child_keys
        .iter()
        .map(|k| k.token().unwrap().text().to_string())
        .collect::<Vec<_>>()
        .join(".");

    Some(CodeAction {
        title: CodeActionRefactorRewriteName::DottedKeysToInlineTable.to_string(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        edit: Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document_uri.to_owned().into(),
                    version: None,
                },
                edits: vec![
                    OneOf::Left(TextEdit {
                        range: tombi_text::Range {
                            start: parent_key.syntax().range().start,
                            end: value.range().start,
                        }
                        .into(),
                        new_text: format!(
                            "{} = {{ {}{}",
                            parent_key.token().unwrap().text(),
                            child_key_text,
                            if child_keys.len() == 1 { " = " } else { "." }
                        ),
                    }),
                    OneOf::Left(TextEdit {
                        range: tombi_text::Range::at(if key_value.trailing_comment().is_some() {
                            // If there's a trailing comment, find where the actual value ends
                            // by looking for the comment start position
                            key_value.trailing_comment().unwrap().syntax().range().start
                        } else {
                            value.range().end
                        })
                        .into(),
                        new_text: " }".to_string(),
                    }),
                ],
            }])),
            change_annotations: None,
        }),
        ..Default::default()
    })
}

pub fn inline_table_to_dot_keys_code_action(
    text_document_uri: &tombi_uri::Uri,
    root: &tombi_ast::Root,
    position: tombi_text::Position,
    _accessors: &[Accessor],
    _contexts: &[AccessorContext],
) -> Option<CodeAction> {
    // Find the KeyValue node at the position
    let key_value = find_key_value_at_position(root, position)?;
    tracing::debug!(
        "Found key_value for inline table conversion: {:?}",
        key_value.syntax().range()
    );

    // Check if the value is an inline table
    let value = key_value.value()?;
    tracing::debug!("Found value: {:?}", value);
    let inline_table = match value {
        tombi_ast::Value::InlineTable(table) => {
            tracing::debug!("Found inline table");
            table
        }
        _ => {
            tracing::debug!("Value is not an inline table");
            return None;
        }
    };

    // Check if the inline table has only one key-value pair
    let key_values: Vec<_> = inline_table.key_values().collect();
    if key_values.len() != 1 {
        return None;
    }

    let child_key_value = key_values.first()?;
    let child_key = child_key_value.keys()?;
    let child_value = child_key_value.value()?;

    // Check if there are no comments that would make conversion unsafe
    if !inline_table.inner_begin_dangling_comments().is_empty()
        || !inline_table.inner_end_dangling_comments().is_empty()
        || child_key.leading_comments().next().is_some()
        || child_value.trailing_comment().is_some()
    {
        return None;
    }

    // Find the parent key context by looking at the key_value node
    let parent_key = key_value.keys()?;
    let parent_key_first = parent_key.keys().next()?;

    // Build the child key string (e.g., "bar.baz" for nested keys)
    let _child_key_text = child_key
        .keys()
        .map(|k| k.token().unwrap().text().to_string())
        .collect::<Vec<_>>()
        .join(".");

    Some(CodeAction {
        title: CodeActionRefactorRewriteName::InlineTableToDottedKeys.to_string(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        edit: Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: text_document_uri.to_owned().into(),
                    version: None,
                },
                edits: vec![
                    OneOf::Left(TextEdit {
                        range: tombi_text::Range::new(
                            parent_key_first.syntax().range().end,
                            child_key.syntax().range().start,
                        )
                        .into(),
                        new_text: ".".to_string(),
                    }),
                    OneOf::Left(TextEdit {
                        range: tombi_text::Range::new(
                            child_value.range().end,
                            if key_value.trailing_comment().is_some() {
                                // If there's a trailing comment, only remove up to the comment
                                key_value.trailing_comment().unwrap().syntax().range().start
                            } else {
                                inline_table.range().end
                            },
                        )
                        .into(),
                        new_text: "".to_string(),
                    }),
                ],
            }])),
            change_annotations: None,
        }),
        ..Default::default()
    })
}

// Helper function to find KeyValue node at a specific position
fn find_key_value_at_position(
    root: &tombi_ast::Root,
    position: tombi_text::Position,
) -> Option<tombi_ast::KeyValue> {
    let mut found_key_values = Vec::new();

    for node in ancestors_at_position(root.syntax(), position) {
        if let Some(key_value) = tombi_ast::KeyValue::cast(node.to_owned()) {
            found_key_values.push(key_value);
        }
    }

    // Return the outermost KeyValue (last in the list)
    found_key_values.into_iter().last()
}
