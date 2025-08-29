use tower_lsp::lsp_types::{CompletionTextEdit, InsertTextFormat, TextEdit, Url};

use crate::completion::completion_hint::{AddLeadingComma, AddTrailingComma};

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
                    new_text: format!(" = {label}"),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: None,
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InArray {
                add_leading_comma,
                add_trailing_comma,
            }) => {
                let new_text = match add_trailing_comma {
                    Some(_) => format!("${{1:{label}}},$0"),
                    None => format!("${{0:{label}}}"),
                };
                let additional_text_edits =
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, position);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        range: tombi_text::Range::at(position).into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::InTableHeader | CompletionHint::Comma { .. }) | None => None,
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
            Some(CompletionHint::InArray {
                add_leading_comma,
                add_trailing_comma,
            }) => {
                let new_text = match add_trailing_comma {
                    Some(_) => format!("{quote}$1{quote},$0"),
                    None => format!("{quote}$1{quote}$0"),
                };
                let additional_text_edits =
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, position);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        range: tombi_text::Range::at(position).into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::InTableHeader | CompletionHint::Comma { .. }) | None => {
                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text: format!("{quote}$1{quote}$0"),
                        range: tombi_text::Range::at(position).into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits: None,
                })
            }
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
            Some(CompletionHint::InArray {
                add_leading_comma,
                add_trailing_comma,
            }) => {
                let new_text = match add_trailing_comma {
                    Some(_) => format!("[$1],$0"),
                    None => format!("[$1]$0"),
                };
                let additional_text_edits =
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, position);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        range: tombi_text::Range::at(position).into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::InTableHeader | CompletionHint::Comma { .. }) | None => {
                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text: "[$1]$0".to_string(),
                        range: tombi_text::Range::at(position).into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits: None,
                })
            }
        }
    }

    pub fn new_inline_table(
        position: tombi_text::Position,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { range, .. } | CompletionHint::EqualTrigger { range },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: " = {{ $1 }}$0".to_string(),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    range: range.into(),
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InArray {
                add_leading_comma,
                add_trailing_comma,
            }) => {
                let new_text = match add_trailing_comma {
                    Some(_) => format!("{{ $1 }},$0"),
                    None => format!("{{ $1 }}$0"),
                };
                let additional_text_edits =
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, position);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        range: tombi_text::Range::at(position).into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::InTableHeader) => None,
            Some(CompletionHint::Comma { .. }) | None => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: "{{ $1 }}$0".to_string(),
                    range: tombi_text::Range::at(position).into(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: None,
            }),
        }
    }

    pub fn new_key(
        key_name: &str,
        key_range: tombi_text::Range,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(CompletionHint::InArray {
                add_leading_comma,
                add_trailing_comma,
            }) => {
                let new_text = match add_trailing_comma {
                    Some(_) => format!("{{ {key_name}$1 }},$0"),
                    None => format!("{{ {key_name}$1 }}$0"),
                };
                let additional_text_edits =
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, key_range.start);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        range: key_range.into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
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
            Some(CompletionHint::InTableHeader | CompletionHint::Comma { .. }) | None => None,
        }
    }

    pub fn new_additional_key(
        key_name: &str,
        key_range: tombi_text::Range,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(CompletionHint::InArray {
                add_leading_comma,
                add_trailing_comma,
            }) => {
                let new_text = match add_trailing_comma {
                    Some(_) => format!("{{ ${{1:{key_name}}} }},$0"),
                    None => format!("{{ ${{1:{key_name}}} }}$0"),
                };
                let additional_text_edits =
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, key_range.start);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        range: key_range.into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::EqualTrigger { range, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {{ ${{1:{key_name}}} }}$0"),
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
            Some(CompletionHint::InTableHeader | CompletionHint::Comma { .. }) | None => {
                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text: format!("${{0:{key_name}}}"),
                        range: key_range.into(),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits: None,
                })
            }
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

    pub fn new_schema_comment_directive(
        position: tombi_text::Position,
        comment_range: tombi_text::Range,
        text_document_uri: &Url,
    ) -> Option<Self> {
        let file_name = std::path::Path::new(text_document_uri.path())
            .file_stem() // "ccc"
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        let schema_uri = format!("https://json.schemastore.org/{file_name}.json",);

        Some(Self {
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: format!("#:schema ${{0:{schema_uri}}}"),
                range: tombi_text::Range::at(position).into(),
            }),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            additional_text_edits: Some(vec![TextEdit {
                range: comment_range.into(),
                new_text: "".to_string(),
            }]),
        })
    }

    pub fn new_comment_directive(
        directive_name: &str,
        position: tombi_text::Position,
        comment_range: tombi_text::Range,
    ) -> Option<Self> {
        Some(Self {
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: format!("#:{directive_name} "),
                range: tombi_text::Range::at(position).into(),
            }),
            insert_text_format: None,
            additional_text_edits: Some(vec![TextEdit {
                range: comment_range.into(),
                new_text: "".to_string(),
            }]),
        })
    }

    pub fn with_position(mut self, position: tombi_text::Position) -> Self {
        fn offset(
            range: tower_lsp::lsp_types::Range,
            position: tombi_text::Position,
        ) -> tower_lsp::lsp_types::Range {
            let mut start = range.start;
            start.line += position.line;
            start.character += position.column;
            let mut end = range.end;
            end.line += position.line;
            end.character += position.column;

            tower_lsp::lsp_types::Range { start, end }
        }

        self.text_edit = match self.text_edit {
            CompletionTextEdit::Edit(text_edit) => CompletionTextEdit::Edit(TextEdit {
                range: offset(text_edit.range, position),
                new_text: text_edit.new_text,
            }),
            CompletionTextEdit::InsertAndReplace(insert_replace_edit) => {
                CompletionTextEdit::InsertAndReplace(tower_lsp::lsp_types::InsertReplaceEdit {
                    insert: offset(insert_replace_edit.insert, position),
                    replace: offset(insert_replace_edit.replace, position),
                    new_text: insert_replace_edit.new_text,
                })
            }
        };

        self.additional_text_edits = self.additional_text_edits.map(|mut edits| {
            edits.iter_mut().for_each(|edit| {
                edit.range = offset(edit.range, position);
            });
            edits
        });

        self
    }
}

fn head_comma_text_edits(
    add_leading_comma: Option<AddLeadingComma>,
    _add_trailing_comma: Option<AddTrailingComma>,
    cursor_position: tombi_text::Position,
) -> Option<Vec<TextEdit>> {
    if let Some(AddLeadingComma { start_position }) = add_leading_comma {
        let new_text = if start_position.line == cursor_position.line {
            ", ".to_string()
        } else {
            format!(
                ",{newlines}{spaces}",
                newlines = "\n"
                    .repeat((cursor_position.line.saturating_sub(start_position.line)) as usize),
                spaces = " ".repeat(cursor_position.column as usize)
            )
        };

        Some(vec![TextEdit {
            range: tombi_text::Range {
                start: start_position,
                end: cursor_position,
            }
            .into(),
            new_text,
        }])
    } else {
        None
    }
}
