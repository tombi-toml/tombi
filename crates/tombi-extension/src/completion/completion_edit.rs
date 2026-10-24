use crate::{
    TextEdit,
    completion::completion_hint::{AddLeadingComma, AddTrailingComma},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertTextFormat {
    PlainText,
    Snippet,
}

impl InsertTextFormat {
    pub const PLAIN_TEXT: Self = Self::PlainText;
    pub const SNIPPET: Self = Self::Snippet;
}

use super::CompletionHint;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionTextEdit {
    Edit(TextEdit),
    InsertAndReplace(InsertReplaceEdit),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertReplaceEdit {
    pub new_text: String,
    pub insert: tombi_text::Span,
    pub replace: tombi_text::Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionEdit {
    pub text_edit: CompletionTextEdit,
    pub insert_text_format: Option<InsertTextFormat>,
    pub additional_text_edits: Option<Vec<TextEdit>>,
}

impl CompletionEdit {
    pub fn new_literal(
        label: &str,
        offset: tombi_text::Offset,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { cleanup_span, .. }
                | CompletionHint::EqualTrigger { cleanup_span, .. },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {label}"),
                    span: tombi_text::Span::empty(offset),
                }),
                insert_text_format: None,
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
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
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, offset);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        span: tombi_text::Span::empty(offset),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::InTableHeader { .. } | CompletionHint::Comma { .. }) | None => {
                None
            }
        }
    }

    pub fn new_selectable_literal(
        label: &str,
        offset: tombi_text::Offset,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { cleanup_span, .. }
                | CompletionHint::EqualTrigger { cleanup_span, .. },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = ${{0:{label}}}"),
                    span: tombi_text::Span::empty(offset),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
                    new_text: "".to_string(),
                }]),
            }),
            _ => None,
        }
    }

    pub fn new_string_literal(
        quote: &str,
        offset: tombi_text::Offset,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { cleanup_span, .. }
                | CompletionHint::EqualTrigger { cleanup_span, .. },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {quote}$1{quote}$0"),
                    span: tombi_text::Span::empty(offset),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
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
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, offset);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        span: tombi_text::Span::empty(offset),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::InTableHeader { .. } | CompletionHint::Comma { .. }) | None => {
                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text: format!("{quote}$1{quote}$0"),
                        span: tombi_text::Span::empty(offset),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits: None,
                })
            }
        }
    }

    pub fn new_string_literal_while_editing(
        label: &str,
        value_span: tombi_text::Span,
    ) -> Option<Self> {
        Some(Self {
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: label.to_string(),
                span: value_span,
            }),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            additional_text_edits: None,
        })
    }

    pub fn new_array_literal(
        offset: tombi_text::Offset,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { cleanup_span, .. }
                | CompletionHint::EqualTrigger { cleanup_span, .. },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: " = [$1]$0".to_string(),
                    span: tombi_text::Span::empty(offset),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InArray {
                add_leading_comma,
                add_trailing_comma,
            }) => {
                let new_text = match add_trailing_comma {
                    Some(_) => "[$1],$0".to_string(),
                    None => "[$1]$0".to_string(),
                };
                let additional_text_edits =
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, offset);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        span: tombi_text::Span::empty(offset),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::InTableHeader { .. } | CompletionHint::Comma { .. }) | None => {
                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text: "[$1]$0".to_string(),
                        span: tombi_text::Span::empty(offset),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits: None,
                })
            }
        }
    }

    pub fn new_inline_table(
        offset: tombi_text::Offset,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(
                CompletionHint::DotTrigger { cleanup_span, .. }
                | CompletionHint::EqualTrigger { cleanup_span, .. },
            ) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: " = { $1 }$0".to_string(),
                    span: tombi_text::Span::empty(offset),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InArray {
                add_leading_comma,
                add_trailing_comma,
            }) => {
                let new_text = match add_trailing_comma {
                    Some(_) => "{ $1 },$0".to_string(),
                    None => "{ $1 }$0".to_string(),
                };
                let additional_text_edits =
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, offset);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        span: tombi_text::Span::empty(offset),
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::InTableHeader { .. }) => None,
            Some(CompletionHint::Comma { .. }) | None => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: "{ $1 }$0".to_string(),
                    span: tombi_text::Span::empty(offset),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: None,
            }),
        }
    }

    pub fn new_key(
        key_name: &str,
        key_span: tombi_text::Span,
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
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, key_span.start);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        span: key_span,
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::EqualTrigger { cleanup_span, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {{ {key_name}$1 }}$0"),
                    span: tombi_text::Span::empty(cleanup_span.end),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::DotTrigger { cleanup_span, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(".{key_name}"),
                    span: tombi_text::Span::empty(cleanup_span.end),
                }),
                insert_text_format: None,
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InTableHeader { .. } | CompletionHint::Comma { .. }) | None => {
                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text: key_name.to_string(),
                        span: key_span,
                    }),
                    insert_text_format: None,
                    additional_text_edits: None,
                })
            }
        }
    }

    pub fn new_key_with_literal(
        key_name: &str,
        key_span: tombi_text::Span,
        value_label: &str,
        completion_hint: Option<CompletionHint>,
    ) -> Option<Self> {
        match completion_hint {
            Some(CompletionHint::InArray {
                add_leading_comma,
                add_trailing_comma,
            }) => {
                let new_text = match add_trailing_comma {
                    Some(_) => format!("{{ {key_name} = {value_label} }},$0"),
                    None => format!("{{ {key_name} = {value_label} }}$0"),
                };
                let additional_text_edits =
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, key_span.start);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        span: key_span,
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::EqualTrigger { cleanup_span, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {{ {key_name} = {value_label} }}"),
                    span: tombi_text::Span::empty(cleanup_span.end),
                }),
                insert_text_format: None,
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::DotTrigger { cleanup_span, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(".{key_name} = {value_label}"),
                    span: tombi_text::Span::empty(cleanup_span.end),
                }),
                insert_text_format: None,
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::Comma { .. }) | None => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!("{key_name} = {value_label}"),
                    span: key_span,
                }),
                insert_text_format: None,
                additional_text_edits: None,
            }),
            Some(CompletionHint::InTableHeader { .. }) => None,
        }
    }

    pub fn new_additional_key(
        key_name: &str,
        key_span: tombi_text::Span,
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
                    head_comma_text_edits(add_leading_comma, add_trailing_comma, key_span.start);

                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text,
                        span: key_span,
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits,
                })
            }
            Some(CompletionHint::EqualTrigger { cleanup_span, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(" = {{ ${{1:{key_name}}} }}$0"),
                    span: tombi_text::Span::empty(cleanup_span.end),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::DotTrigger { cleanup_span, .. }) => Some(Self {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    new_text: format!(".${{0:{key_name}}}"),
                    span: tombi_text::Span::empty(cleanup_span.end),
                }),
                insert_text_format: None,
                additional_text_edits: Some(vec![TextEdit {
                    span: cleanup_span,
                    new_text: "".to_string(),
                }]),
            }),
            Some(CompletionHint::InTableHeader { .. } | CompletionHint::Comma { .. }) | None => {
                Some(Self {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        new_text: format!("${{0:{key_name}}}"),
                        span: key_span,
                    }),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    additional_text_edits: None,
                })
            }
        }
    }

    pub fn new_magic_trigger(trigger: &str, offset: tombi_text::Offset) -> Option<Self> {
        Some(Self {
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: trigger.to_string(),
                span: tombi_text::Span::empty(offset),
            }),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            additional_text_edits: None,
        })
    }

    pub fn new_schema_comment_directive(
        offset: tombi_text::Offset,
        comment_span: tombi_text::Span,
        text_document_uri: &tombi_uri::Uri,
    ) -> Option<Self> {
        let file_name = std::path::Path::new(text_document_uri.path())
            .file_stem() // "ccc"
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        let schema_uri = format!(
            "https://{}/{}.json",
            tombi_uri::schemastore_hostname!(),
            file_name
        );

        Some(Self {
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: format!("#:schema ${{0:{schema_uri}}}"),
                span: tombi_text::Span::empty(offset),
            }),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            additional_text_edits: Some(vec![TextEdit {
                span: comment_span,
                new_text: "".to_string(),
            }]),
        })
    }

    pub fn new_comment_directive(
        directive_name: &str,
        offset: tombi_text::Offset,
        comment_span: tombi_text::Span,
    ) -> Option<Self> {
        Some(Self {
            text_edit: CompletionTextEdit::Edit(TextEdit {
                new_text: format!("#:{directive_name} "),
                span: tombi_text::Span::empty(offset),
            }),
            insert_text_format: None,
            additional_text_edits: Some(vec![TextEdit {
                span: comment_span,
                new_text: "".to_string(),
            }]),
        })
    }

    /// Shifts the edits by `offset`, for an edit computed in a text embedded at `offset`.
    pub fn with_offset(mut self, offset: tombi_text::Offset) -> Self {
        self.text_edit = match self.text_edit {
            CompletionTextEdit::Edit(text_edit) => CompletionTextEdit::Edit(TextEdit {
                span: text_edit.span + offset,
                new_text: text_edit.new_text,
            }),
            CompletionTextEdit::InsertAndReplace(insert_replace_edit) => {
                CompletionTextEdit::InsertAndReplace(InsertReplaceEdit {
                    insert: insert_replace_edit.insert + offset,
                    replace: insert_replace_edit.replace + offset,
                    new_text: insert_replace_edit.new_text,
                })
            }
        };

        if let Some(edits) = &mut self.additional_text_edits {
            for edit in edits {
                edit.span += offset;
            }
        }

        self
    }
}

fn head_comma_text_edits(
    add_leading_comma: Option<AddLeadingComma>,
    _add_trailing_comma: Option<AddTrailingComma>,
    cursor_offset: tombi_text::Offset,
) -> Option<Vec<TextEdit>> {
    if let Some(AddLeadingComma {
        start_offset,
        line_breaks,
        indent,
    }) = add_leading_comma
    {
        let new_text = if line_breaks == 0 {
            ", ".to_string()
        } else {
            format!(
                ",{newlines}{spaces}",
                newlines = "\n".repeat(line_breaks as usize),
                spaces = " ".repeat(indent as usize)
            )
        };

        Some(vec![TextEdit {
            span: tombi_text::Span::new(start_offset, cursor_offset),
            new_text,
        }])
    } else {
        None
    }
}
