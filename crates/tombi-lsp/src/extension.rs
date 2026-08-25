use tombi_text::IntoLsp as _;

pub trait IntoLsp {
    type Lsp;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp;
}

impl IntoLsp for tombi_extension::Location {
    type Lsp = tower_lsp::lsp_types::Location;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        Self::Lsp::new(self.uri.into(), self.range.into_lsp(line_index))
    }
}

impl IntoLsp for tombi_extension::DocumentLink {
    type Lsp = tower_lsp::lsp_types::DocumentLink;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        Self::Lsp {
            range: self.range.into_lsp(line_index),
            target: Some(self.target.into()),
            tooltip: Some(self.tooltip.into_owned()),
            data: None,
        }
    }
}

impl IntoLsp for tombi_extension::InlayHint {
    type Lsp = tower_lsp::lsp_types::InlayHint;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        Self::Lsp {
            position: self.position.into_lsp(line_index),
            label: self.label.into(),
            kind: self.kind.map(|kind| match kind {
                tombi_extension::InlayHintKind::Type => tower_lsp::lsp_types::InlayHintKind::TYPE,
                tombi_extension::InlayHintKind::Parameter => {
                    tower_lsp::lsp_types::InlayHintKind::PARAMETER
                }
            }),
            text_edits: None,
            tooltip: self.tooltip.map(Into::into),
            padding_left: self.padding_left,
            padding_right: self.padding_right,
            data: None,
        }
    }
}

impl IntoLsp for tombi_extension::TextEdit {
    type Lsp = tower_lsp::lsp_types::TextEdit;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        Self::Lsp {
            range: self.range.into_lsp(line_index),
            new_text: self.new_text,
        }
    }
}

impl IntoLsp for tombi_extension::InsertReplaceEdit {
    type Lsp = tower_lsp::lsp_types::InsertReplaceEdit;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        Self::Lsp {
            new_text: self.new_text,
            insert: self.insert.into_lsp(line_index),
            replace: self.replace.into_lsp(line_index),
        }
    }
}

impl IntoLsp for tombi_extension::CompletionTextEdit {
    type Lsp = tower_lsp::lsp_types::CompletionTextEdit;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        match self {
            tombi_extension::CompletionTextEdit::Edit(edit) => {
                Self::Lsp::Edit(edit.into_lsp_type(line_index))
            }
            tombi_extension::CompletionTextEdit::InsertAndReplace(edit) => {
                Self::Lsp::InsertAndReplace(edit.into_lsp_type(line_index))
            }
        }
    }
}

impl IntoLsp for tombi_extension::CompletionContent {
    type Lsp = tower_lsp::lsp_types::CompletionItem;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        use tombi_extension::CompletionContentPriority as Priority;
        use tower_lsp::lsp_types::{
            CompletionItem, CompletionItemLabelDetails, Documentation, InsertTextMode,
            MarkupContent, MarkupKind,
        };

        let sorted_text = format!("{}_{}", self.priority.as_prefix(), self.label);
        let omit_detail = self.documentation.is_some()
            && matches!(
                self.priority,
                Priority::TypeHint | Priority::TypeHintTrue | Priority::TypeHintFalse
            );

        let schema_text = self.schema_uri.as_ref().and_then(|schema_uri| {
            tombi_schema_store::get_schema_name(schema_uri)
                .map(|name| format!("Schema: [{name}]({schema_uri})\n"))
        });
        let documentation = match self.documentation {
            Some(documentation) => {
                let mut documentation = documentation.trim_end().to_string();
                if let Some(schema_text) = schema_text {
                    documentation.push_str("\n\n");
                    documentation.push_str(&schema_text);
                }
                Some(documentation)
            }
            None => schema_text,
        };

        let (insert_text_format, text_edit, additional_text_edits) = match self.edit {
            Some(edit) => (
                edit.insert_text_format.map(|format| match format {
                    tombi_extension::InsertTextFormat::PlainText => {
                        tower_lsp::lsp_types::InsertTextFormat::PLAIN_TEXT
                    }
                    tombi_extension::InsertTextFormat::Snippet => {
                        tower_lsp::lsp_types::InsertTextFormat::SNIPPET
                    }
                }),
                Some(edit.text_edit.into_lsp_type(line_index)),
                edit.additional_text_edits.map(|edits| {
                    edits
                        .into_iter()
                        .map(|edit| edit.into_lsp_type(line_index))
                        .collect()
                }),
            ),
            None => (None, None, None),
        };

        let label_details = match self.priority {
            Priority::Custom(_) => Some(CompletionItemLabelDetails {
                detail: None,
                description: self.detail.clone(),
            }),
            Priority::Default => Some(CompletionItemLabelDetails {
                detail: None,
                description: Some(match &self.detail {
                    Some(detail) => format!("[Default] {detail}"),
                    None => "Default".to_string(),
                }),
            }),
            Priority::Const => label_details("Const", &self.detail),
            Priority::Enum => label_details("Enum", &self.detail),
            Priority::Example => label_details("Example", &self.detail),
            Priority::Key => Some(CompletionItemLabelDetails {
                detail: None,
                description: self.detail.clone(),
            }),
            Priority::OptionalKey | Priority::AdditionalKey => Some(CompletionItemLabelDetails {
                detail: Some("?".to_string()),
                description: self.detail.clone(),
            }),
            Priority::TypeHint
            | Priority::TypeHintKey
            | Priority::TypeHintTrue
            | Priority::TypeHintFalse => Some(CompletionItemLabelDetails {
                detail: None,
                description: Some(match &self.detail {
                    Some(detail) if !detail.trim().is_empty() => detail.clone(),
                    _ => "Type Hint".to_string(),
                }),
            }),
        }
        .map(|mut details| {
            if let Some(emoji_icon) = self.emoji_icon {
                details.description = Some(format!(
                    "{} {}",
                    emoji_icon,
                    details.description.unwrap_or_default()
                ));
            }
            details
        });

        CompletionItem {
            label: self.label,
            label_details,
            kind: Some(completion_kind(self.kind)),
            detail: if omit_detail {
                None
            } else {
                self.detail.map(|detail| match self.emoji_icon {
                    Some(emoji_icon) => format!("{emoji_icon} {detail}"),
                    None => detail,
                })
            },
            documentation: documentation.map(|value| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value,
                })
            }),
            sort_text: Some(sorted_text),
            filter_text: self.filter_text,
            insert_text_format,
            text_edit,
            insert_text_mode: Some(InsertTextMode::ADJUST_INDENTATION),
            additional_text_edits,
            preselect: self.preselect,
            deprecated: self.deprecated,
            ..Default::default()
        }
    }
}

impl IntoLsp for tombi_extension::CodeActionOrCommand {
    type Lsp = tower_lsp::lsp_types::CodeActionOrCommand;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        match self {
            tombi_extension::CodeActionOrCommand::CodeAction(action) => {
                Self::Lsp::CodeAction(action.into_lsp_type(line_index))
            }
            tombi_extension::CodeActionOrCommand::Command(command) => {
                Self::Lsp::Command(tower_lsp::lsp_types::Command {
                    title: command.title,
                    command: command.command,
                    arguments: command.arguments,
                })
            }
        }
    }
}

impl IntoLsp for tombi_extension::CodeAction {
    type Lsp = tower_lsp::lsp_types::CodeAction;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        Self::Lsp {
            title: self.title,
            kind: self.kind.map(|kind| match kind {
                tombi_extension::CodeActionKind::RefactorRewrite => {
                    tower_lsp::lsp_types::CodeActionKind::REFACTOR_REWRITE
                }
            }),
            edit: self.edit.map(|edit| edit.into_lsp_type(line_index)),
            disabled: self
                .disabled
                .map(|disabled| tower_lsp::lsp_types::CodeActionDisabled {
                    reason: disabled.reason,
                }),
            ..Default::default()
        }
    }
}

impl IntoLsp for tombi_extension::WorkspaceEdit {
    type Lsp = tower_lsp::lsp_types::WorkspaceEdit;

    fn into_lsp_type(self, line_index: &tombi_text::LineIndex) -> Self::Lsp {
        Self::Lsp {
            changes: self.changes.map(|changes| {
                changes
                    .into_iter()
                    .map(|(uri, document_edits)| {
                        (
                            uri.into(),
                            document_edits
                                .edits
                                .into_iter()
                                .map(|edit| edit.into_lsp_type(&document_edits.line_index))
                                .collect(),
                        )
                    })
                    .collect()
            }),
            document_changes: self.document_changes.map(|changes| match changes {
                tombi_extension::DocumentChanges::Edits(edits) => {
                    tower_lsp::lsp_types::DocumentChanges::Edits(
                        edits
                            .into_iter()
                            .map(|edit| edit.into_lsp_type(line_index))
                            .collect(),
                    )
                }
            }),
            change_annotations: None,
        }
    }
}

impl IntoLsp for tombi_extension::TextDocumentEdit {
    type Lsp = tower_lsp::lsp_types::TextDocumentEdit;

    fn into_lsp_type(self, _line_index: &tombi_text::LineIndex) -> Self::Lsp {
        Self::Lsp {
            text_document: tower_lsp::lsp_types::OptionalVersionedTextDocumentIdentifier {
                uri: self.text_document.uri.into(),
                version: self.text_document.version,
            },
            edits: self
                .edits
                .into_iter()
                .map(|edit| match edit {
                    tombi_extension::OneOf::Left(edit) => {
                        tower_lsp::lsp_types::OneOf::Left(edit.into_lsp_type(&self.line_index))
                    }
                    tombi_extension::OneOf::Right(edit) => tower_lsp::lsp_types::OneOf::Right(
                        tower_lsp::lsp_types::AnnotatedTextEdit {
                            text_edit: edit.text_edit.into_lsp_type(&self.line_index),
                            annotation_id: edit.annotation_id,
                        },
                    ),
                })
                .collect(),
        }
    }
}

fn label_details(
    default: &str,
    detail: &Option<String>,
) -> Option<tower_lsp::lsp_types::CompletionItemLabelDetails> {
    Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
        detail: None,
        description: Some(detail.clone().unwrap_or_else(|| default.to_string())),
    })
}

fn completion_kind(
    kind: tombi_extension::CompletionKind,
) -> tower_lsp::lsp_types::CompletionItemKind {
    use tombi_extension::CompletionKind as Kind;
    use tower_lsp::lsp_types::CompletionItemKind;

    match kind {
        Kind::Boolean => CompletionItemKind::CONSTANT,
        Kind::Integer | Kind::Float => CompletionItemKind::VALUE,
        Kind::String => CompletionItemKind::TEXT,
        Kind::Enum => CompletionItemKind::ENUM_MEMBER,
        Kind::OffsetDateTime | Kind::LocalDateTime | Kind::LocalDate | Kind::LocalTime => {
            CompletionItemKind::EVENT
        }
        Kind::Array | Kind::Table => CompletionItemKind::STRUCT,
        Kind::Key => CompletionItemKind::FIELD,
        Kind::MagicTrigger => CompletionItemKind::METHOD,
        Kind::CommentDirective => CompletionItemKind::KEYWORD,
        Kind::File => CompletionItemKind::FILE,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tombi_extension::{
        DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier, TextDocumentEdit,
        TextEdit, WorkspaceEdit,
    };
    use tombi_text::{EncodingKind, LineIndex, Position, Range};

    use super::IntoLsp;

    #[test]
    fn workspace_edit_uses_each_target_document_line_index() {
        let workspace_uri = tombi_uri::Uri::from_str("file:///workspace.toml").unwrap();
        let member_uri = tombi_uri::Uri::from_str("file:///member.toml").unwrap();
        let workspace_line_index = LineIndex::new("e\u{301}x", EncodingKind::Utf16);
        let member_line_index = LineIndex::new("👨‍👩‍👧‍👦x", EncodingKind::Utf16);
        let edit = WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![
                TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: workspace_uri,
                        version: None,
                    },
                    line_index: workspace_line_index,
                    edits: vec![OneOf::Left(TextEdit {
                        range: Range::at(Position::new(0, 1)),
                        new_text: String::new(),
                    })],
                },
                TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier {
                        uri: member_uri,
                        version: None,
                    },
                    line_index: member_line_index,
                    edits: vec![OneOf::Left(TextEdit {
                        range: Range::at(Position::new(0, 1)),
                        new_text: String::new(),
                    })],
                },
            ])),
        };

        let fallback = LineIndex::new("a", EncodingKind::Utf16);
        let Some(tower_lsp::lsp_types::DocumentChanges::Edits(edits)) =
            edit.into_lsp_type(&fallback).document_changes
        else {
            panic!("expected document edits");
        };
        let positions: Vec<_> = edits
            .into_iter()
            .map(|edit| match &edit.edits[0] {
                tower_lsp::lsp_types::OneOf::Left(edit) => edit.range.start.character,
                tower_lsp::lsp_types::OneOf::Right(_) => unreachable!(),
            })
            .collect();

        assert_eq!(positions, [2, 11]);
    }
}
