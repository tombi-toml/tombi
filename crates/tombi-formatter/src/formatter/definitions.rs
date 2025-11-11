use tombi_config::{DateTimeDelimiter, FormatOptions, IndentStyle, QuoteStyle};

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
    pub indent_width: u8,
    pub trailing_comment_space: String,
    pub quote_style: QuoteStyle,
    pub date_time_delimiter: Option<&'static str>,
    pub array_bracket_space: String,
    pub array_element_space: String,
    pub inline_table_brace_space: String,
    pub inline_table_element_space: String,
}

impl FormatDefinitions {
    pub fn new(options: &FormatOptions) -> Self {
        Self {
            line_width: options.line_width.unwrap_or_default().value(),
            line_ending: options.line_ending.unwrap_or_default().into(),
            indent_style: options.indent_style.unwrap_or_default(),
            indent_width: options.indent_width.unwrap_or_default().value(),
            trailing_comment_space: " ".repeat(
                options
                    .trailing_comment_space_width
                    .unwrap_or_default()
                    .value() as usize,
            ),
            quote_style: options.quote_style.unwrap_or_default(),
            date_time_delimiter: match options.date_time_delimiter.unwrap_or_default() {
                DateTimeDelimiter::T => Some("T"),
                DateTimeDelimiter::Space => Some(" "),
                DateTimeDelimiter::Preserve => None,
            },
            array_bracket_space: " ".repeat(
                options
                    .array_bracket_space_width
                    .unwrap_or_default()
                    .value() as usize,
            ),
            array_element_space: " ".repeat(
                options
                    .array_element_space_width
                    .unwrap_or_default()
                    .value() as usize,
            ),
            inline_table_brace_space: " ".repeat(
                options
                    .inline_table_brace_space_width
                    .unwrap_or_default()
                    .value() as usize,
            ),
            inline_table_element_space: " ".repeat(
                options
                    .inline_table_element_space_width
                    .unwrap_or_default()
                    .value() as usize,
            ),
        }
    }
}
