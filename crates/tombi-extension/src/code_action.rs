use std::collections::HashMap;

use serde_json::Value;
use tombi_diagnostic::Diagnostic;
use tombi_text::{FromLsp, IntoLsp};
use tower_lsp::lsp_types::{CodeActionDisabled, CodeActionKind, Command};

use crate::{TextDocumentEdit, TextEdit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedTextEdit {
    pub text_edit: TextEdit,
    pub annotation_id: String,
}

impl FromLsp<AnnotatedTextEdit> for tower_lsp::lsp_types::AnnotatedTextEdit {
    fn from_lsp(
        source: AnnotatedTextEdit,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::AnnotatedTextEdit {
        tower_lsp::lsp_types::AnnotatedTextEdit {
            text_edit: source.text_edit.into_lsp(line_index),
            annotation_id: source.annotation_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentChanges {
    Edits(Vec<TextDocumentEdit>),
}

impl FromLsp<DocumentChanges> for tower_lsp::lsp_types::DocumentChanges {
    fn from_lsp(
        source: DocumentChanges,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::DocumentChanges {
        match source {
            DocumentChanges::Edits(edits) => tower_lsp::lsp_types::DocumentChanges::Edits(
                edits
                    .into_iter()
                    .map(|edit| edit.into_lsp(line_index))
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChangeAnnotation {
    pub label: String,
    pub needs_confirmation: Option<bool>,
    pub description: Option<String>,
}

impl From<ChangeAnnotation> for tower_lsp::lsp_types::ChangeAnnotation {
    fn from(value: ChangeAnnotation) -> Self {
        Self {
            label: value.label,
            needs_confirmation: value.needs_confirmation,
            description: value.description,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkspaceEdit {
    pub changes: Option<HashMap<tombi_uri::Uri, Vec<TextEdit>>>,
    pub document_changes: Option<DocumentChanges>,
    pub change_annotations: Option<HashMap<String, ChangeAnnotation>>,
}

impl FromLsp<WorkspaceEdit> for tower_lsp::lsp_types::WorkspaceEdit {
    fn from_lsp(
        source: WorkspaceEdit,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::WorkspaceEdit {
        tower_lsp::lsp_types::WorkspaceEdit {
            changes: source.changes.map(|changes| {
                changes
                    .into_iter()
                    .map(|(uri, edits)| {
                        (
                            uri.into(),
                            edits
                                .into_iter()
                                .map(|edit| edit.into_lsp(line_index))
                                .collect(),
                        )
                    })
                    .collect()
            }),
            document_changes: source
                .document_changes
                .map(|changes| changes.into_lsp(line_index)),
            change_annotations: source.change_annotations.map(|annotations| {
                annotations
                    .into_iter()
                    .map(|(id, annotation)| (id, annotation.into()))
                    .collect()
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<CodeActionKind>,
    pub diagnostics: Option<Vec<Diagnostic>>,
    pub edit: Option<WorkspaceEdit>,
    pub command: Option<Command>,
    pub is_preferred: Option<bool>,
    pub disabled: Option<CodeActionDisabled>,
    pub data: Option<Value>,
}

impl FromLsp<CodeAction> for tower_lsp::lsp_types::CodeAction {
    fn from_lsp(
        source: CodeAction,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::CodeAction {
        tower_lsp::lsp_types::CodeAction {
            title: source.title,
            kind: source.kind,
            diagnostics: source.diagnostics.map(|diagnostics| {
                diagnostics
                    .into_iter()
                    .map(|diagnostic| diagnostic.into_lsp(line_index))
                    .collect()
            }),
            edit: source.edit.map(|edit| edit.into_lsp(line_index)),
            command: source.command,
            is_preferred: source.is_preferred,
            disabled: source.disabled,
            data: source.data,
        }
    }
}

impl FromLsp<CodeAction> for tower_lsp::lsp_types::CodeActionOrCommand {
    fn from_lsp(
        source: CodeAction,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::CodeActionOrCommand {
        tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(source.into_lsp(line_index))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeActionOrCommand {
    CodeAction(Box<CodeAction>),
    Command(Box<Command>),
}

impl FromLsp<CodeActionOrCommand> for tower_lsp::lsp_types::CodeActionOrCommand {
    fn from_lsp(
        source: CodeActionOrCommand,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::CodeActionOrCommand {
        match source {
            CodeActionOrCommand::CodeAction(action) => {
                tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(
                    (*action).into_lsp(line_index),
                )
            }
            CodeActionOrCommand::Command(command) => {
                tower_lsp::lsp_types::CodeActionOrCommand::Command(*command)
            }
        }
    }
}
