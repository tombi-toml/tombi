use tombi_ast::AstNode;
use tombi_document_tree::{dig_accessors, TableKind};
use tombi_schema_store::{Accessor, AccessorContext, AccessorKeyKind};
use tombi_text::IntoLsp;
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
    line_index: &tombi_text::LineIndex,
    _root: &tombi_ast::Root,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
) -> Option<CodeAction> {
    if accessors.len() < 2 {
        return None;
    }
    debug_assert!(accessors.len() == contexts.len());
    let AccessorContext::Key(parent_key_context) = &contexts[accessors.len() - 2] else {
        return None;
    };

    let (accessor, value) = dig_accessors(document_tree, &accessors[..accessors.len() - 1])?;

    match (accessor, value) {
        (Accessor::Key(parent_key), tombi_document_tree::Value::Table(table))
            if table.len() == 1
                && matches!(
                    parent_key_context.kind,
                    AccessorKeyKind::Dotted | AccessorKeyKind::KeyValue
                )
                && !matches!(table.kind(), TableKind::InlineTable { .. }) =>
        {
            let (key, value) = table.key_values().iter().next().unwrap();

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
                                    start: parent_key_context.range.start,
                                    end: value.range().start,
                                }
                                .into_lsp(line_index),
                                new_text: format!(
                                    "{} = {{ {}{}",
                                    parent_key,
                                    key.value,
                                    if table.kind() == TableKind::KeyValue {
                                        " = "
                                    } else {
                                        "."
                                    }
                                ),
                            }),
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range::at(value.symbol_range().end)
                                    .into_lsp(line_index),
                                new_text: " }".to_string(),
                            }),
                        ],
                    }])),
                    change_annotations: None,
                }),
                ..Default::default()
            })
        }
        _ => None,
    }
}

pub fn inline_table_to_dot_keys_code_action(
    text_document_uri: &tombi_uri::Uri,
    line_index: &tombi_text::LineIndex,
    root: &tombi_ast::Root,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    contexts: &[AccessorContext],
) -> Option<CodeAction> {
    if accessors.len() < 2 {
        return None;
    }
    debug_assert!(accessors.len() == contexts.len());
    let AccessorContext::Key(parent_context) = &contexts[accessors.len() - 2] else {
        return None;
    };

    let (_, value) = dig_accessors(document_tree, &accessors[..accessors.len() - 1])?;

    match value {
        tombi_document_tree::Value::Table(table)
            if table.len() == 1
                && matches!(table.kind(), TableKind::InlineTable { has_comment: false }) =>
        {
            let Some(node) = get_ast_inline_table_node(root, table) else {
                return None;
            };
            if !node.inner_begin_dangling_comments().is_empty()
                || node
                    .inner_end_dangling_comments()
                    .into_iter()
                    .flatten()
                    .next()
                    .is_some()
                || node.has_inner_comments()
            {
                return None;
            }
            let Some((key, value)) = table.key_values().iter().next() else {
                return None;
            };

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
                                    parent_context.range.end,
                                    key.range().start,
                                )
                                .into_lsp(line_index),
                                new_text: ".".to_string(),
                            }),
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range::new(
                                    value.range().end,
                                    table.symbol_range().end,
                                )
                                .into_lsp(line_index),
                                new_text: "".to_string(),
                            }),
                        ],
                    }])),
                    change_annotations: None,
                }),
                ..Default::default()
            })
        }
        _ => None,
    }
}

fn get_ast_inline_table_node(
    root: &tombi_ast::Root,
    table: &tombi_document_tree::Table,
) -> Option<tombi_ast::InlineTable> {
    let target_range = table.range();
    for node in root.syntax().descendants() {
        if let Some(inline_table) = tombi_ast::InlineTable::cast(node) {
            if inline_table.range() == target_range {
                return Some(inline_table);
            }
        }
    }
    None
}
