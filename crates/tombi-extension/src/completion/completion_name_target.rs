use super::{CompletionEdit, CompletionHint, CompletionTextEdit};
use crate::TextEdit;

/// Where a completed name is written.
#[derive(Debug, Clone, Copy)]
pub enum CompletionNameTarget<'a> {
    /// Replaces an existing string value. `prefix` is the text typed so far.
    Replace {
        span: tombi_text::Span,
        prefix: Option<&'a str>,
    },
    /// Inserts at an empty value position.
    Insert {
        offset: tombi_text::Offset,
        hint: Option<CompletionHint>,
    },
}

impl<'a> CompletionNameTarget<'a> {
    pub fn prefix(&self) -> Option<&'a str> {
        match self {
            Self::Replace { prefix, .. } => *prefix,
            Self::Insert { .. } => None,
        }
    }

    /// `label` must already be quoted when the value is a string.
    pub fn edit(&self, label: &str) -> Option<CompletionEdit> {
        match self {
            Self::Replace { span, .. } => {
                CompletionEdit::new_string_literal_while_editing(label, *span)
            }
            // `new_literal` has no edit without a hint, so a plain insertion is built here.
            Self::Insert { offset, hint: None } => Some(CompletionEdit {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: label.to_owned(),
                    span: tombi_text::Span::empty(*offset),
                }),
                insert_text_format: None,
                additional_text_edits: None,
            }),
            Self::Insert {
                offset,
                hint: Some(hint),
            } => CompletionEdit::new_literal(label, *offset, Some(*hint)),
        }
    }
}
