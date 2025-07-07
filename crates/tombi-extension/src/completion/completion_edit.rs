use tower_lsp::lsp_types::{CompletionTextEdit, InsertTextFormat, TextEdit, Url};

use super::CompletionHint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionEdit {
    pub text_edit: tower_lsp::lsp_types::CompletionTextEdit,
    pub insert_text_format: Option<tower_lsp::lsp_types::InsertTextFormat>,
    pub additional_text_edits: Option<Vec<tower_lsp::lsp_types::TextEdit>>,
}

impl CompletionEdit {
    pub fn new_literal(
        label: &str,
        position: tombi_text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {}", label),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: None,
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
        position: tombi_text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = ${{0:{label}}}"),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
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
        position: tombi_text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {quote}$1{quote}$0"),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InArray | CompletionHint::InTableHeader) | None => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("{quote}$1{quote}$0"),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: None,
            }),
        }
    }

    pub fn new_string_literal_while_editing(
        label: &str,
        value_range: tombi_text::Range,
    ) -> Option<Self> {
        Some(Self {
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: label.to_string(),
                range: value_range.into(),
            }),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            additional_text_edits: None,
        })
    }

    pub fn new_array_literal(
        position: tombi_text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. }
                | CompletionHint::EqualTrigger { range, .. },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: " = [$1]$0".to_string(),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InArray | CompletionHint::InTableHeader) | None => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: "[$1]$0".to_string(),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: None,
            }),
        }
    }

    pub fn new_inline_table(
        position: tombi_text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(CompletionHint::DotTrigger { range, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: " = { $1 }$0".to_string(),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InTableHeader) => None,
            Some(CompletionHint::InArray | CompletionHint::EqualTrigger { .. }) | None => {
                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text: "{ $1 }$0".to_string(),
                        range: tombi_text::Range::at(position).into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits: None,
                })
            }
        }
    }

    pub fn new_key(
        key_name: &str,
        key_range: tombi_text::Range,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(CompletionHint::InArray) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("{{ {key_name}$1 }}$0"),
                    range: key_range.into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: None,
            }),
            Some(CompletionHint::EqualTrigger { range, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {{ {key_name}$1 }}$0"),
                    range: key_range.into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::DotTrigger { range, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(".{key_name}"),
                    range: key_range.into(),
                }),
                insert_text_format: None,
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InTableHeader) | None => None,
        }
    }

    pub fn new_additional_key(
        key_name: &str,
        key_range: tombi_text::Range,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(CompletionHint::InArray) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("{{ ${{0:{key_name}}} }}"),
                    range: key_range.into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: None,
            }),
            Some(CompletionHint::EqualTrigger { range, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {{ ${{0:{key_name}}} }}"),
                    range: range.into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::DotTrigger { range, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(".${{0:{key_name}}}"),
                    range: range.into(),
                }),
                insert_text_format: None,
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InTableHeader) | None => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("${{0:{key_name}}}"),
                    range: key_range.into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: None,
            }),
        }
    }

    pub fn new_magic_trigger(trigger: &str, position: tombi_text::Position) -> Option<Self> {
        Some(Self {
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: trigger.to_string(),
                range: tombi_text::Range::at(position).into(),
            }),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            additional_text_edits: None,
        })
    }

    pub fn new_comment_schema_directive(
        position: tombi_text::Position,
        prefix_range: tombi_text::Range,
        text_document_uri: &Url,
    ) -> Option<Self> {
        let file_name = std::path::Path::new(text_document_uri.path())
            .file_stem() // "ccc"
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        let schema_url = format!("https://json.schemastore.org/{file_name}.json",);

        Some(Self {
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: format!("#:schema ${{0:{schema_url}}}"),
                range: tombi_text::Range::at(position).into(),
            }),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            additional_text_edits: Some(vec![TextEdit {
                range: prefix_range.into(),
                new_text: "".to_string(),
            }]),
        })
    }
}
