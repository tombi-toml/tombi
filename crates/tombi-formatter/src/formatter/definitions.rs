use tombi_config::{DateTimeDelimiter, FormatOptions, IndentStyle, StringQuoteStyle};

/// FormatDefinitions provides the definition of the format that does not have the freedom set by [`FormatOptions`][crate::FormatOptions].
///
/// NOTE: Some of the items defined in FormatDefinitions may be moved to [`FormatOptions`][crate::FormatOptions] in the future.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone)]
pub struct FormatDefinitions {
    pub line_width: u8,
    pub line_ending: &'static str,
    pub indent_style: IndentStyle,
    pub indent_sub_tables: bool,
    pub indent_table_key_values: bool,
    pub indent_width: u8,
    pub string_quote_style: StringQuoteStyle,
    pub trailing_comment_alignment: bool,
    pub trailing_comment_space: String,
    pub key_value_equal_alignment: bool,
    pub key_value_equal_space: String,
    pub date_time_delimiter: Option<&'static str>,
    pub array_bracket_space: String,
    pub array_comma_space: String,
    pub inline_table_brace_space: String,
    pub inline_table_comma_space: String,
}

impl FormatDefinitions {
    pub fn new(options: &FormatOptions) -> Self {
        Self {
            line_width: options
                .rules
                .as_ref()
                .and_then(|rules| rules.line_width)
                .unwrap_or_default()
                .value(),
            line_ending: options
                .rules
                .as_ref()
                .and_then(|rules| rules.line_ending)
                .unwrap_or_default()
                .into(),
            indent_style: options
                .rules
                .as_ref()
                .and_then(|rules| rules.indent_style)
                .unwrap_or_default(),
            indent_sub_tables: options
                .rules
                .as_ref()
                .and_then(|rules| rules.indent_sub_tables)
                .unwrap_or_default(),
            indent_table_key_values: options
                .rules
                .as_ref()
                .and_then(|rules| rules.indent_table_key_value_pairs)
                .unwrap_or_default(),
            indent_width: options
                .rules
                .as_ref()
                .and_then(|rules| rules.indent_width)
                .unwrap_or_default()
                .value(),
            trailing_comment_alignment: options
                .rules
                .as_ref()
                .and_then(|rules| rules.trailing_comment_alignment)
                .unwrap_or_default(),
            trailing_comment_space: " ".repeat(
                options
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.trailing_comment_space_width)
                    .unwrap_or_default()
                    .value() as usize,
            ),
            key_value_equal_alignment: options
                .rules
                .as_ref()
                .and_then(|rules| rules.key_value_equals_sign_alignment)
                .unwrap_or_default(),
            key_value_equal_space: " ".repeat(
                options
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.key_value_equals_sign_space_width)
                    .unwrap_or_default()
                    .value() as usize,
            ),
            string_quote_style: options
                .rules
                .as_ref()
                .and_then(|rules| rules.string_quote_style)
                .unwrap_or_default(),
            date_time_delimiter: match options
                .rules
                .as_ref()
                .and_then(|rules| rules.date_time_delimiter)
                .unwrap_or_default()
            {
                DateTimeDelimiter::T => Some("T"),
                DateTimeDelimiter::Space => Some(" "),
                DateTimeDelimiter::Preserve => None,
            },
            array_bracket_space: " ".repeat(
                options
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.array_bracket_space_width)
                    .unwrap_or_default()
                    .value() as usize,
            ),
            array_comma_space: " ".repeat(
                options
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.array_comma_space_width)
                    .unwrap_or_default()
                    .value() as usize,
            ),
            inline_table_brace_space: " ".repeat(
                options
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.inline_table_brace_space_width)
                    .unwrap_or_default()
                    .value() as usize,
            ),
            inline_table_comma_space: " ".repeat(
                options
                    .rules
                    .as_ref()
                    .and_then(|rules| rules.inline_table_comma_space_width)
                    .unwrap_or_default()
                    .value() as usize,
            ),
        }
    }
}
