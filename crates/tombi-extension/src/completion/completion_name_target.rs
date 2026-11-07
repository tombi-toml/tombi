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
    /// Inserts an item into a list. A comma is written on a side only when that side has an item and no comma yet.
    ListItem {
        offset: tombi_text::Offset,
        leading_comma: bool,
        trailing_comma: bool,
    },
}

impl<'a> CompletionNameTarget<'a> {
    pub fn prefix(&self) -> Option<&'a str> {
        match self {
            Self::Replace { prefix, .. } => *prefix,
            Self::Insert { .. } | Self::ListItem { .. } => None,
        }
    }

    /// The target for inserting into a list at `offset`. `item_spans` are the spans of all items in the list.
    pub fn list_insert(
        offset: tombi_text::Offset,
        hint: Option<CompletionHint>,
        item_spans: impl IntoIterator<Item = tombi_text::Span>,
    ) -> Self {
        let (has_leading_comma, has_trailing_comma) = match hint {
            Some(CompletionHint::Comma {
                leading_comma,
                trailing_comma,
            }) => (leading_comma.is_some(), trailing_comma.is_some()),
            None => (false, false),
            hint => return Self::Insert { offset, hint },
        };
        let (has_previous_item, has_next_item) = item_spans
            .into_iter()
            .fold((false, false), |(previous, next), span| {
                (previous || span.end <= offset, next || span.start >= offset)
            });
        Self::ListItem {
            offset,
            leading_comma: has_previous_item && !has_leading_comma,
            trailing_comma: has_next_item && !has_trailing_comma,
        }
    }

    /// The string item the cursor is strictly inside. A cursor on the item's boundary
    /// inserts next to the item, so that item is not counted as the one being edited.
    pub fn edited_span(&self, offset: tombi_text::Offset) -> Option<tombi_text::Span> {
        match self {
            Self::Replace { span, .. } if span.start < offset && offset < span.end => Some(*span),
            _ => None,
        }
    }

    /// `label` must already be quoted when the value is a string.
    pub fn edit(&self, label: &str) -> Option<CompletionEdit> {
        match self {
            Self::Replace { span, .. } => {
                CompletionEdit::new_string_literal_while_editing(label, *span)
            }
            // `new_literal` has no edit without a hint, so a plain insertion is built here.
            Self::Insert { offset, hint: None } => Some(plain_insert(label, *offset)),
            Self::ListItem {
                offset,
                leading_comma,
                trailing_comma,
            } => {
                let leading = if *leading_comma { ", " } else { "" };
                let trailing = if *trailing_comma { ", " } else { "" };
                Some(plain_insert(
                    &format!("{leading}{label}{trailing}"),
                    *offset,
                ))
            }
            Self::Insert {
                offset,
                hint: Some(hint),
            } => CompletionEdit::new_literal(label, *offset, Some(*hint)),
        }
    }
}

fn plain_insert(new_text: &str, offset: tombi_text::Offset) -> CompletionEdit {
    CompletionEdit {
        text_edit: CompletionTextEdit::Edit(TextEdit {
            new_text: new_text.to_owned(),
            span: tombi_text::Span::empty(offset),
        }),
        insert_text_format: None,
        additional_text_edits: None,
    }
}
