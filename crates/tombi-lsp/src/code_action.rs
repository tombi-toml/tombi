use tombi_document_tree::TableKind;
use tombi_schema_store::{dig_accessors, Accessor, AccessorContext, AccessorKeyKind};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier,
    TextDocumentEdit, TextDocumentIdentifier, TextEdit, WorkspaceEdit,
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
    text_document: &TextDocumentIdentifier,
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
                ) =>
        {
            let (key, value) = table.key_values().iter().next().unwrap();

            if table.kind() == TableKind::InlineTable {
                return None;
            }

            Some(CodeAction {
                title: CodeActionRefactorRewriteName::DottedKeysToInlineTable.to_string(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE),
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document.clone().uri,
                            version: None,
                        },
                        edits: vec![
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range {
                                    start: parent_key_context.range.start,
                                    end: value.range().start,
                                }
                                .into(),
                                new_text: format!(
                                    "{} = {{ {}{}",
                                    parent_key,
                                    key.value(),
                                    if table.kind() == TableKind::KeyValue {
                                        " = "
                                    } else {
                                        "."
                                    }
                                ),
                            }),
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range::at(value.symbol_range().end).into(),
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
    text_document: &TextDocumentIdentifier,
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
            if table.len() == 1 && table.kind() == TableKind::InlineTable =>
        {
            let (key, value) = table.key_values().iter().next().unwrap();

            Some(CodeAction {
                title: CodeActionRefactorRewriteName::InlineTableToDottedKeys.to_string(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE),
                edit: Some(WorkspaceEdit {
                    changes: None,
                    document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                        text_document: OptionalVersionedTextDocumentIdentifier {
                            uri: text_document.uri.clone(),
                            version: None,
                        },
                        edits: vec![
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range::new(
                                    parent_context.range.end,
                                    key.range().start,
                                )
                                .into(),
                                new_text: ".".to_string(),
                            }),
                            OneOf::Left(TextEdit {
                                range: tombi_text::Range::new(
                                    value.range().end,
                                    table.symbol_range().end,
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
        _ => None,
    }
}
