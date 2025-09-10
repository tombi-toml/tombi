mod completion_edit;
mod completion_hint;
mod completion_kind;

use std::ops::Deref;

pub use completion_edit::CompletionEdit;
pub use completion_hint::{AddLeadingComma, AddTrailingComma, CommaHint, CompletionHint};
pub use completion_kind::CompletionKind;
use tombi_schema_store::{get_schema_name, SchemaUri};

use crate::get_tombi_github_uri;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum CompletionContentPriority {
    Custom(String),
    Default,
    Const,
    Enum,
    Key,
    OptionalKey,
    AdditionalKey,
    TypeHint,
    TypeHintKey,
    TypeHintTrue,
    TypeHintFalse,
}

impl CompletionContentPriority {
    pub fn as_prefix(&self) -> String {
        match self {
            CompletionContentPriority::Custom(value) => value.to_string(),
            // NOTE: 30 is the prefix for completion items from extensions
            //       that should be prioritized over basic features.
            CompletionContentPriority::Default => "50".to_string(),
            CompletionContentPriority::Const => "51".to_string(),
            CompletionContentPriority::Enum => "52".to_string(),
            CompletionContentPriority::Key => "53".to_string(),
            CompletionContentPriority::OptionalKey => "54".to_string(),
            CompletionContentPriority::AdditionalKey => "55".to_string(),
            CompletionContentPriority::TypeHint => "56".to_string(),
            CompletionContentPriority::TypeHintKey => "57".to_string(),
            CompletionContentPriority::TypeHintTrue => "58".to_string(),
            CompletionContentPriority::TypeHintFalse => "59".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentContext {
    DocumentDirective(tombi_ast::Comment),
    ValueDirective(tombi_ast::Comment),
    Normal(tombi_ast::Comment),
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
    pub schema_uri: Option<SchemaUri>,
    pub deprecated: Option<bool>,
    pub edit: Option<CompletionEdit>,
    pub preselect: Option<bool>,
    pub in_comment: bool,
}

impl CompletionContent {
    pub fn new_const_value(
        kind: CompletionKind,
        label: String,
        detail: Option<String>,
        documentation: Option<String>,
        edit: Option<CompletionEdit>,
        schema_uri: Option<&SchemaUri>,
        deprecated: Option<bool>,
    ) -> Self {
        Self {
            label: label.clone(),
            kind,
            emoji_icon: None,
            priority: CompletionContentPriority::Const,
            detail,
            documentation,
            filter_text: None,
            schema_uri: schema_uri.cloned(),
            edit,
            deprecated,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_enumerate_value(
        kind: CompletionKind,
        label: String,
        detail: Option<String>,
        documentation: Option<String>,
        edit: Option<CompletionEdit>,
        schema_uri: Option<&SchemaUri>,
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
            schema_uri: schema_uri.cloned(),
            edit,
            deprecated,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_default_value(
        kind: CompletionKind,
        label: String,
        detail: Option<String>,
        documentation: Option<String>,
        edit: Option<CompletionEdit>,
        schema_uri: Option<&SchemaUri>,
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
            schema_uri: schema_uri.cloned(),
            edit,
            deprecated,
            preselect: Some(true),
            in_comment: false,
        }
    }

    pub fn new_type_hint_value(
        kind: CompletionKind,
        label: impl Into<String>,
        detail: impl Into<String>,
        edit: Option<CompletionEdit>,
        schema_uri: Option<&SchemaUri>,
    ) -> Self {
        Self {
            label: label.into(),
            kind,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::TypeHint,
            detail: Some(detail.into()),
            documentation: None,
            filter_text: None,
            schema_uri: schema_uri.cloned(),
            edit,
            deprecated: None,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_type_hint_boolean(
        value: bool,
        edit: Option<CompletionEdit>,
        schema_uri: Option<&SchemaUri>,
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
            schema_uri: schema_uri.cloned(),
            edit,
            deprecated: None,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_type_hint_string(
        kind: CompletionKind,
        quote: char,
        detail: impl Into<String>,
        edit: Option<CompletionEdit>,
        schema_uri: Option<&SchemaUri>,
    ) -> Self {
        Self {
            label: format!("{quote}{quote}"),
            kind,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::TypeHint,
            detail: Some(detail.into()),
            documentation: None,
            filter_text: None,
            schema_uri: schema_uri.cloned(),
            edit,
            deprecated: None,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_type_hint_inline_table(
        position: tombi_text::Position,
        schema_uri: Option<&SchemaUri>,
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
            schema_uri: schema_uri.cloned(),
            edit: CompletionEdit::new_inline_table(position, completion_hint),
            deprecated: None,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_type_hint_key(
        key_name: &str,
        key_range: tombi_text::Range,
        schema_uri: Option<&SchemaUri>,
        completion_hint: Option<CompletionHint>,
    ) -> Self {
        let edit = CompletionEdit::new_key(key_name, key_range, completion_hint);

        Self {
            label: "$key".to_string(),
            kind: CompletionKind::Table,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::TypeHintKey,
            detail: Some("Key".to_string()),
            documentation: None,
            filter_text: Some(key_name.to_string()),
            schema_uri: schema_uri.cloned(),
            edit,
            deprecated: None,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_type_hint_empty_key(
        position: tombi_text::Position,
        schema_uri: Option<&SchemaUri>,
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
            schema_uri: schema_uri.cloned(),
            deprecated: None,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_key(
        key_name: &str,
        position: tombi_text::Position,
        detail: Option<String>,
        documentation: Option<String>,
        required_keys: Option<&Vec<String>>,
        schema_uri: Option<&SchemaUri>,
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
            schema_uri: schema_uri.cloned(),
            deprecated,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_pattern_key(
        patterns: &[String],
        position: tombi_text::Position,
        schema_uri: Option<&SchemaUri>,
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
                    documentation.push_str(&format!("- `{pattern}`\n"));
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
            schema_uri: schema_uri.cloned(),
            deprecated: None,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_additional_key(
        position: tombi_text::Position,
        schema_uri: Option<&SchemaUri>,
        deprecated: Option<bool>,
        completion_hint: Option<CompletionHint>,
    ) -> Self {
        Self {
            label: "$key".to_string(),
            kind: CompletionKind::Key,
            emoji_icon: None,
            priority: CompletionContentPriority::AdditionalKey,
            detail: Some("Additional Key".to_string()),
            documentation: None,
            filter_text: None,
            edit: CompletionEdit::new_additional_key(
                "key",
                tombi_text::Range::at(position),
                completion_hint,
            ),
            schema_uri: schema_uri.cloned(),
            deprecated,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn new_magic_triggers(
        key: &str,
        position: tombi_text::Position,
        schema_uri: Option<&SchemaUri>,
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
                schema_uri: schema_uri.cloned(),
                deprecated: None,
                preselect: None,
                in_comment: false,
            })
            .collect()
    }

    /// Creates a new schema comment directive completion content.
    ///
    /// NOTE: schema directive is formatted to follow Taplo's format.
    ///       If Taplo didn't exist, it would be formatted as `# schema: ${1:url}`.
    ///
    ///       See: https://taplo.tamasfe.dev/configuration/directives.html#the-schema-directive
    ///
    /// ```toml
    /// #:schema https://...
    /// ```
    pub fn new_comment_directive(
        directive_name: &str,
        detail: impl Into<String>,
        documentation: impl Into<String>,
        edit: Option<CompletionEdit>,
    ) -> Self {
        Self {
            label: directive_name.to_string(),
            kind: CompletionKind::CommentDirective,
            emoji_icon: Some('🦅'),
            priority: CompletionContentPriority::Key,
            detail: Some(detail.into()),
            documentation: Some(documentation.into()),
            filter_text: None,
            edit,
            schema_uri: None,
            deprecated: None,
            preselect: None,
            in_comment: false,
        }
    }

    pub fn with_position(mut self, position: tombi_text::Position) -> Self {
        self.edit = self.edit.map(|edit| edit.with_position(position));
        self
    }
}

impl From<CompletionContent> for tower_lsp::lsp_types::CompletionItem {
    fn from(completion_content: CompletionContent) -> Self {
        const SECTION_SEPARATOR: &str = "-----";

        let sorted_text = format!(
            "{}_{}",
            completion_content.priority.as_prefix(),
            &completion_content.label
        );

        let mut schema_text = None;
        if let Some(schema_uri) = &completion_content.schema_uri {
            let schema_uri = match get_tombi_github_uri(schema_uri) {
                Some(schema_uri) => schema_uri,
                None => schema_uri.deref().clone(),
            };
            if let Some(schema_filename) = get_schema_name(&schema_uri) {
                schema_text = Some(format!("Schema: [{schema_filename}]({schema_uri})\n"));
            }
        }
        let documentation = match completion_content.documentation {
            Some(documentation) => {
                let mut documentation = documentation;
                if let Some(schema_text) = schema_text {
                    documentation.push_str(&format!("\n\n{SECTION_SEPARATOR}\n\n"));
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
            CompletionContentPriority::Custom(_) => {
                Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                    detail: None,
                    description: completion_content.detail.clone(),
                })
            }
            CompletionContentPriority::Default => {
                Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                    detail: None,
                    description: Some(match &completion_content.detail {
                        Some(detail) => format!("[Default] {detail}"),
                        None => "Default".to_string(),
                    }),
                })
            }
            CompletionContentPriority::Const => {
                Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                    detail: None,
                    description: Some(match &completion_content.detail {
                        Some(detail) => detail.to_string(),
                        None => "Const".to_string(),
                    }),
                })
            }
            CompletionContentPriority::Enum => {
                Some(tower_lsp::lsp_types::CompletionItemLabelDetails {
                    detail: None,
                    description: Some(match &completion_content.detail {
                        Some(detail) => detail.to_string(),
                        None => "Enum".to_string(),
                    }),
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
                    description: Some(match &completion_content.detail {
                        Some(detail) if !detail.trim().is_empty() => detail.clone(),
                        _ => "Type Hint".to_string(),
                    }),
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
                    format!("{emoji_icon} {detail}")
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
