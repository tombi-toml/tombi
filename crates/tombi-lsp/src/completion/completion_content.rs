use tombi_config::TomlVersion;
use tombi_schema_store::{get_schema_name, SchemaUrl};

use super::{completion_edit::CompletionEdit, completion_kind::CompletionKind, CompletionHint};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CompletionContentPriority {
    Default = 0,
    Enum,
    Key,
    OptionalKey,
    AdditionalKey,
    TypeHint,
    TypeHintKey,
    TypeHintTrue,
    TypeHintFalse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionContent {
    pub label: String,
    pub kind: CompletionKind,
    pub emoji_icon: Option<char>,
    pub priority: CompletionContentPriority,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub filter_text: Option<String>,
    pub schema_url: Option<SchemaUrl>,
    pub deprecated: Option<bool>,
    pub edit: Option<CompletionEdit>,
    pub preselect: Option<bool>,
}

impl CompletionContent {
    pub fn new_enumerate_value(
        kind: CompletionKind,
        label: String,
        detail: Option<String>,
        documentation: Option<String>,
        edit: Option<CompletionEdit>,
        schema_url: Option<&SchemaUrl>,
        deprecated: Option<bool>,
    ) -> Self {
        Self {
            label: label.clone(),
            kind,
            emoji_icon: None,
            priority: CompletionContentPriority::Enum,
            detail,
            documentation,
            filter_text: None,
            schema_url: schema_url.cloned(),
            edit,
            deprecated,
            preselect: None,
        }
    }

    pub fn new_default_value(
        kind: CompletionKind,
        label: String,
        detail: Option<String>,
        documentation: Option<String>,
        edit: Option<CompletionEdit>,
        schema_url: Option<&SchemaUrl>,
        deprecated: Option<bool>,
    ) -> Self {
        Self {
            label,
            kind,
            emoji_icon: None,
            priority: CompletionContentPriority::Default,
            detail,
            documentation,
            filter_text: None,
            schema_url: schema_url.cloned(),
            edit,
            deprecated,
            preselect: Some(true),
        }
    }

    pub fn new_type_hint_value(
        kind: CompletionKind,
        label: impl Into<String>,
        detail: impl Into<String>,
        edit: Option<CompletionEdit>,
        schema_url: Option<&SchemaUrl>,
    ) -> Self {
        Self {
            label: label.into(),
            kind,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::TypeHint,
            detail: Some(detail.into()),
            documentation: None,
            filter_text: None,
            schema_url: schema_url.cloned(),
            edit,
            deprecated: None,
            preselect: None,
        }
    }

    pub fn new_type_hint_boolean(
        value: bool,
        edit: Option<CompletionEdit>,
        schema_url: Option<&SchemaUrl>,
    ) -> Self {
        Self {
            label: value.to_string(),
            kind: CompletionKind::Boolean,
            emoji_icon: Some('🦅'),
            priority: if value {
                CompletionContentPriority::TypeHintTrue
            } else {
                CompletionContentPriority::TypeHintFalse
            },
            detail: Some("Boolean".to_string()),
            documentation: None,
            filter_text: None,
            schema_url: schema_url.cloned(),
            edit,
            deprecated: None,
            preselect: None,
        }
    }

    pub fn new_type_hint_string(
        kind: CompletionKind,
        quote: char,
        detail: impl Into<String>,
        edit: Option<CompletionEdit>,
        schema_url: Option<&SchemaUrl>,
    ) -> Self {
        Self {
            label: format!("{}{}", quote, quote),
            kind,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::TypeHint,
            detail: Some(detail.into()),
            documentation: None,
            filter_text: None,
            schema_url: schema_url.cloned(),
            edit,
            deprecated: None,
            preselect: None,
        }
    }

    pub fn new_type_hint_inline_table(
        position: tombi_text::Position,
        schema_url: Option<&SchemaUrl>,
        completion_hint: Option<CompletionHint>,
    ) -> Self {
        Self {
            label: "{}".to_string(),
            kind: CompletionKind::Table,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::TypeHint,
            detail: Some("InlineTable".to_string()),
            documentation: None,
            filter_text: None,
            schema_url: schema_url.cloned(),
            edit: CompletionEdit::new_inline_table(position, completion_hint),
            deprecated: None,
            preselect: None,
        }
    }

    pub fn new_type_hint_key(
        key: &tombi_document_tree::Key,
        toml_version: TomlVersion,
        schema_url: Option<&SchemaUrl>,
        completion_hint: Option<CompletionHint>,
    ) -> Self {
        let key_name = key.to_raw_text(toml_version);
        let edit = CompletionEdit::new_key(&key_name, key.range(), completion_hint);

        Self {
            label: "$key".to_string(),
            kind: CompletionKind::Table,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::TypeHintKey,
            detail: Some("Key".to_string()),
            documentation: None,
            filter_text: Some(key_name),
            schema_url: schema_url.cloned(),
            edit,
            deprecated: None,
            preselect: None,
        }
    }

    pub fn new_type_hint_empty_key(
        position: tombi_text::Position,
        schema_url: Option<&SchemaUrl>,
        completion_hint: Option<CompletionHint>,
    ) -> Self {
        Self {
            label: "$key".to_string(),
            kind: CompletionKind::Key,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::TypeHintKey,
            detail: Some("Key".to_string()),
            documentation: None,
            filter_text: None,
            edit: CompletionEdit::new_additional_key(
                "key",
                tombi_text::Range::at(position),
                completion_hint,
            ),
            schema_url: schema_url.cloned(),
            deprecated: None,
            preselect: None,
        }
    }

    pub fn new_key(
        key_name: &str,
        position: tombi_text::Position,
        detail: Option<String>,
        documentation: Option<String>,
        required_keys: Option<&Vec<String>>,
        schema_url: Option<&SchemaUrl>,
        deprecated: Option<bool>,
        completion_hint: Option<CompletionHint>,
    ) -> Self {
        let label = key_name.to_string();
        let required = required_keys
            .map(|required_keys| required_keys.contains(&label))
            .unwrap_or_default();

        let key_range = match completion_hint {
            Some(
                CompletionHint::DotTrigger { range } | CompletionHint::EqualTrigger { range, .. },
            ) => tombi_text::Range::new(range.end, position),
            _ => tombi_text::Range::at(position),
        };

        Self {
            label,
            kind: CompletionKind::Key,
            emoji_icon: None,
            priority: if required {
                CompletionContentPriority::Key
            } else {
                CompletionContentPriority::OptionalKey
            },
            detail,
            documentation,
            filter_text: None,
            edit: CompletionEdit::new_key(key_name, key_range, completion_hint),
            schema_url: schema_url.cloned(),
            deprecated,
            preselect: None,
        }
    }

    pub fn new_pattern_key(
        patterns: &[String],
        position: tombi_text::Position,
        schema_url: Option<&SchemaUrl>,
        completion_hint: Option<CompletionHint>,
    ) -> Self {
        Self {
            label: "$key".to_string(),
            kind: CompletionKind::Key,
            emoji_icon: None,
            priority: CompletionContentPriority::AdditionalKey,
            detail: Some("Pattern Key".to_string()),
            documentation: if !patterns.is_empty() {
                let mut documentation = "Allowed Patterns:\n\n".to_string();
                for pattern in patterns {
                    documentation.push_str(&format!("- `{}`\n", pattern));
                }
                Some(documentation)
            } else {
                None
            },
            filter_text: None,
            edit: CompletionEdit::new_additional_key(
                "key",
                tombi_text::Range::at(position),
                completion_hint,
            ),
            schema_url: schema_url.cloned(),
            deprecated: None,
            preselect: None,
        }
    }

    pub fn new_additional_key(
        position: tombi_text::Position,
        schema_url: Option<&SchemaUrl>,
        deprecated: Option<bool>,
        completion_hint: Option<CompletionHint>,
    ) -> Self {
        Self {
            label: "$key".to_string(),
            kind: CompletionKind::Key,
            emoji_icon: None,
            priority: CompletionContentPriority::AdditionalKey,
            detail: Some("Additinal Key".to_string()),
            documentation: None,
            filter_text: None,
            edit: CompletionEdit::new_additional_key(
                "key",
                tombi_text::Range::at(position),
                completion_hint,
            ),
            schema_url: schema_url.cloned(),
            deprecated,
            preselect: None,
        }
    }

    pub fn new_magic_triggers(
        key: &str,
        position: tombi_text::Position,
        schema_url: Option<&SchemaUrl>,
    ) -> Vec<Self> {
        [(".", "Dot Trigger"), ("=", "Equal Trigger")]
            .into_iter()
            .map(|(trigger, detail)| Self {
                label: trigger.to_string(),
                kind: CompletionKind::MagicTrigger,
                emoji_icon: Some('🦅'),
                priority: CompletionContentPriority::TypeHint,
                detail: Some(detail.to_string()),
                documentation: None,
                filter_text: Some(format!("{key}{trigger}")),
                edit: CompletionEdit::new_magic_trigger(trigger, position),
                schema_url: schema_url.cloned(),
                deprecated: None,
                preselect: None,
            })
            .collect()
    }

    pub fn new_comment_directive(
        directive: &str,
        detail: impl Into<String>,
        documentation: impl Into<String>,
        edit: Option<CompletionEdit>,
    ) -> Self {
        Self {
            label: directive.to_string(),
            kind: CompletionKind::CommentDirective,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::Key,
            detail: Some(detail.into()),
            documentation: Some(documentation.into()),
            filter_text: None,
            edit,
            schema_url: None,
            deprecated: None,
            preselect: None,
        }
    }
}

impl From<CompletionContent> for tower_lsp::lsp_types::CompletionItem {
    fn from(completion_content: CompletionContent) -> Self {
        const SECTION_SEPARATOR: &str = "-----";

        let sorted_text = format!(
            "{}_{}",
            completion_content.priority as u8, &completion_content.label
        );

        let mut schema_text = None;
        if let Some(schema_url) = &completion_content.schema_url {
            if let Some(schema_filename) = get_schema_name(schema_url) {
                schema_text = Some(format!("Schema: [{schema_filename}]({schema_url})\n"));
            }
        }
        let documentation = match completion_content.documentation {
            Some(documentation) => {
                let mut documentation = documentation;
                if let Some(schema_text) = schema_text {
                    documentation.push_str(&format!("\n\n{}\n\n", SECTION_SEPARATOR));
                    documentation.push_str(&schema_text);
                }
                Some(documentation)
            }
            None => schema_text,
        };

        let (insert_text_format, text_edit, additional_text_edits) = match completion_content.edit {
            Some(edit) => (
                edit.insert_text_format,
                Some(edit.text_edit),
                edit.additional_text_edits,
            ),
            None => (None, None, None),
        };

        let label_details = match completion_content.priority {
            CompletionContentPriority::Default => {
                Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                    detail: None,
                    description: Some("Default".to_string()),
                })
            }
            CompletionContentPriority::Enum => {
                Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                    detail: None,
                    description: Some("Enum".to_string()),
                })
            }
            CompletionContentPriority::Key => {
                Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                    detail: None,
                    description: completion_content.detail.clone(),
                })
            }
            CompletionContentPriority::OptionalKey | CompletionContentPriority::AdditionalKey => {
                Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                    detail: Some("?".to_string()),
                    description: completion_content.detail.clone(),
                })
            }
            CompletionContentPriority::TypeHint
            | CompletionContentPriority::TypeHintKey
            | CompletionContentPriority::TypeHintTrue
            | CompletionContentPriority::TypeHintFalse => {
                Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                    detail: None,
                    description: Some("Type Hint".to_string()),
                })
            }
        }
        .map(|mut details| {
            if let Some(emoji_icon) = completion_content.emoji_icon {
                details.description = Some(format!(
                    "{} {}",
                    emoji_icon,
                    details.description.unwrap_or_default()
                ));
            }
            details
        });

        tower_lsp::lsp_types::CompletionItem {
            label: completion_content.label,
            label_details,
            kind: Some(completion_content.kind.into()),
            detail: completion_content.detail.map(|detail| {
                if let Some(emoji_icon) = completion_content.emoji_icon {
                    format!("{} {}", emoji_icon, detail)
                } else {
                    detail
                }
            }),
            documentation: documentation.map(|documentation| {
                tower_lsp::lsp_types::Documentation::MarkupContent(
                    tower_lsp::lsp_types::MarkupContent {
                        kind: tower_lsp::lsp_types::MarkupKind::Markdown,
                        value: documentation,
                    },
                )
            }),
            sort_text: Some(sorted_text),
            filter_text: completion_content.filter_text,
            insert_text_format,
            text_edit,
            insert_text_mode: Some(tower_lsp::lsp_types::InsertTextMode::ADJUST_INDENTATION),
            additional_text_edits,
            preselect: completion_content.preselect,
            deprecated: completion_content.deprecated,
            ..Default::default()
        }
    }
}
