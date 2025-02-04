use tower_lsp::lsp_types::{CompletionTextEdit, InsertTextFormat, TextEdit};

use super::CompletionHint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionEdit {
    pub insert_text_format: Option<tower_lsp::lsp_types::InsertTextFormat>,
    pub text_edit: tower_lsp::lsp_types::CompletionTextEdit,
    pub additional_text_edits: Option<Vec<tower_lsp::lsp_types::TextEdit>>,
}

impl CompletionEdit {
    pub fn new_literal(
        label: &str,
        position: text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. }
                | CompletionHint::SpaceTrigger { range, .. },
            ) => Some(Self {
                insert_text_format: None,
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {}", label),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            _ => None,
        }
    }

    pub fn new_selectable_literal(
        label: &str,
        position: text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. }
                | CompletionHint::SpaceTrigger { range, .. },
            ) => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = ${{0:{label}}}"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            _ => None,
        }
    }

    pub fn new_string_literal(
        quote: char,
        position: text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. }
                | CompletionHint::SpaceTrigger { range, .. },
            ) => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {quote}$1{quote}$0"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InArray | CompletionHint::InTableHeader) | None => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("{quote}$1{quote}$0"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: None,
            }),
        }
    }

    pub fn new_array_literal(
        position: text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. }
                | CompletionHint::SpaceTrigger { range, .. },
            ) => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: " = [$1]$0".to_string(),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InArray | CompletionHint::InTableHeader) | None => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: "[$1]$0".to_string(),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: None,
            }),
        }
    }

    pub fn new_inline_table(
        position: text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. }
                | CompletionHint::SpaceTrigger { range, .. },
            ) => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: " = { $1 }$0".to_string(),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InTableHeader) => None,
            Some(CompletionHint::InArray | CompletionHint::EqualTrigger { .. }) | None => {
                Some(Self {
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text: "{ $1 }$0".to_string(),
                        range: text::Range::at(position).into(),
                    }),
                    additional_text_edits: None,
                })
            }
        }
    }

    pub fn new_key(
        key_name: &str,
        position: text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(CompletionHint::InArray) => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("{{ {key_name}$1 }}$0"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: None,
            }),
            Some(
                CompletionHint::SpaceTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. },
            ) => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {{ {key_name}$1 }}$0"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::DotTrigger { range, .. }) => Some(Self {
                insert_text_format: None,
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(".{key_name}"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InTableHeader) | None => None,
        }
    }

    pub fn new_additional_propery(
        property_name: &str,
        position: text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(CompletionHint::InArray) => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("{{ ${{0:{property_name}}} }}"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: None,
            }),
            Some(
                CompletionHint::SpaceTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. },
            ) => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {{ ${{0:{property_name}}} }}"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::DotTrigger { range, .. }) => Some(Self {
                insert_text_format: None,
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(".${{0:{property_name}}}"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InTableHeader) | None => Some(Self {
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("${{0:{property_name}}}"),
                    range: text::Range::at(position).into(),
                }),
                additional_text_edits: None,
            }),
        }
    }

    pub fn new_magic_trigger(trigger: &str, position: text::Position) -> Option<Self> {
        Some(Self {
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: trigger.to_string(),
                range: text::Range::at(position).into(),
            }),
            additional_text_edits: None,
        })
    }
}
