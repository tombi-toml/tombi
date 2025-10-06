mod code_action;
mod completion;
mod definition;
mod document_link;
pub use code_action::*;
pub use completion::*;
pub use definition::*;
pub use document_link::*;
use tombi_text::{FromLsp, IntoLsp};
use tower_lsp::lsp_types::OptionalVersionedTextDocumentIdentifier;
pub use tower_lsp::lsp_types::{CodeActionKind, OneOf};

pub trait Extension {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDocumentEdit {
    pub text_document: OptionalVersionedTextDocumentIdentifier,
    pub edits: Vec<OneOf<TextEdit, AnnotatedTextEdit>>,
}

impl FromLsp<TextDocumentEdit> for tower_lsp::lsp_types::TextDocumentEdit {
    fn from_lsp(
        source: TextDocumentEdit,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::TextDocumentEdit {
        tower_lsp::lsp_types::TextDocumentEdit {
            text_document: source.text_document,
            edits: source
                .edits
                .into_iter()
                .map(|edit| match edit {
                    OneOf::Left(edit) => {
                        tower_lsp::lsp_types::OneOf::Left(edit.into_lsp(line_index))
                    }
                    OneOf::Right(edit) => {
                        tower_lsp::lsp_types::OneOf::Right(edit.into_lsp(line_index))
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: tombi_text::Range,
    pub new_text: String,
}

impl FromLsp<TextEdit> for tower_lsp::lsp_types::TextEdit {
    fn from_lsp(
        source: TextEdit,
        line_index: &tombi_text::LineIndex,
    ) -> tower_lsp::lsp_types::TextEdit {
        tower_lsp::lsp_types::TextEdit {
            range: source.range.into_lsp(line_index),
            new_text: source.new_text,
        }
    }
}
