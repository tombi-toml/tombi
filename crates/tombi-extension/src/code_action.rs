use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeActionOrCommand {
    CodeAction(CodeAction),
    Command(Command),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CodeAction {
    pub title: String,
    pub kind: Option<CodeActionKind>,
    pub edit: Option<WorkspaceEdit>,
    pub disabled: Option<CodeActionDisabled>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeActionKind {
    RefactorRewrite,
}

impl CodeActionKind {
    pub const REFACTOR_REWRITE: Self = Self::RefactorRewrite;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeActionDisabled {
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub title: String,
    pub command: String,
    pub arguments: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceEdit {
    pub changes: Option<HashMap<tombi_uri::Uri, DocumentEdits>>,
    pub document_changes: Option<DocumentChanges>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEdits {
    pub line_index: tombi_text::LineIndex,
    pub edits: Vec<crate::TextEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentChanges {
    Edits(Vec<TextDocumentEdit>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDocumentEdit {
    pub text_document: OptionalVersionedTextDocumentIdentifier,
    pub line_index: tombi_text::LineIndex,
    pub edits: Vec<OneOf<crate::TextEdit, AnnotatedTextEdit>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalVersionedTextDocumentIdentifier {
    pub uri: tombi_uri::Uri,
    pub version: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedTextEdit {
    pub text_edit: crate::TextEdit,
    pub annotation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OneOf<L, R> {
    Left(L),
    Right(R),
}
