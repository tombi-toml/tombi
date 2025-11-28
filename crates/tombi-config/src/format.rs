//! Formatting options
//!
//! Options for adjusting the formatting of TOML files.
//! Initially, this structure contained settings related to `line-width`, etc.,
//! but to avoid unnecessary discussions about the format, all settings have been moved to [formatter::FormatDefinition].
//! In the future, there is a possibility that options will be added to this structure,
//! but considering the recent trend of formatters to avoid such discussions by restricting the settings and its results,
//! this structure is currently empty.

use crate::{
    ArrayBracketSpaceWidth, ArrayCommaSpaceWidth, DateTimeDelimiter, IndentStyle, IndentWidth,
    InlineTableBraceSpaceWidth, InlineTableCommaSpaceWidth, KeyValueEqualsSignSpaceWidth,
    LineEnding, LineWidth, StringQuoteStyle, TrailingCommentSpaceWidth,
};

/// # Formatter options
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FormatOptions {
    /// # Format rules
    pub rules: Option<FormatRules>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FormatRules {
    /// # The number of spaces inside the brackets of a single line array.
    ///
    /// ```toml
    /// key = [ 1, 2, 3 ]
    /// #      ^       ^  <- this
    /// ```
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "ArrayBracketSpaceWidth::default")
    )]
    pub array_bracket_space_width: Option<ArrayBracketSpaceWidth>,

    /// # The number of spaces after the comma in a single line array.
    ///
    /// ```toml
    /// key = [ 1, 2, 3 ]
    /// #         ^  ^    <- this
    /// ```
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "ArrayCommaSpaceWidth::default")
    )]
    pub array_comma_space_width: Option<ArrayCommaSpaceWidth>,

    /// # The delimiter between date and time
    ///
    /// In accordance with [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339), you can use `T` or space character between date and time.
    ///
    /// - `T`: Use `T` between date and time like `2001-01-01T00:00:00`
    /// - `space`: Use space between date and time like `2001-01-01 00:00:00`
    /// - `preserve`: Preserve the original delimiter.
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "DateTimeDelimiter::default")
    )]
    pub date_time_delimiter: Option<DateTimeDelimiter>,

    /// # The style of indentation
    ///
    /// Whether to use spaces or tabs for indentation.
    ///
    /// - `space`: Use spaces for indentation.
    /// - `tab`: Use tabs for indentation.
    #[cfg_attr(feature = "jsonschema", schemars(default = "IndentStyle::default"))]
    pub indent_style: Option<IndentStyle>,

    /// # Whether to indent the sub-tables
    ///
    /// If `true`, the sub-table will be indented.
    ///
    /// ```toml
    /// [table]
    ///     [table.sub-table]
    ///     key = "value"
    /// # ^^  <- this
    /// ```
    #[cfg_attr(feature = "jsonschema", schemars(default = "bool::default"))]
    pub indent_sub_tables: Option<bool>,

    /// # Whether to indent the table key-value pairs
    ///
    /// If `true`, the table key-value pairs will be indented.
    ///
    /// ```toml
    /// [table]
    ///     key = "value"
    /// # ^^  <- this
    /// ```
    #[cfg_attr(feature = "jsonschema", schemars(default = "bool::default"))]
    pub indent_table_key_value_pairs: Option<bool>,

    /// # The number of spaces per indentation level
    ///
    /// ⚠️ **WARNING** ⚠️\
    /// This option is only used when the indentation style is `space`.
    #[cfg_attr(feature = "jsonschema", schemars(default = "IndentWidth::default"))]
    pub indent_width: Option<IndentWidth>,

    /// # The number of spaces inside the brackets of a single line inline table.
    ///
    /// ```toml
    /// key = { a = 1, b = 2 }
    /// #      ^            ^  <- this
    /// ```
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "InlineTableBraceSpaceWidth::default")
    )]
    pub inline_table_brace_space_width: Option<InlineTableBraceSpaceWidth>,

    /// # The number of spaces after the comma in a single line inline table.
    ///
    /// ```toml
    /// key = { a = 1, b = 2 }
    /// #             ^  <- this
    /// ```
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "InlineTableCommaSpaceWidth::default")
    )]
    pub inline_table_comma_space_width: Option<InlineTableCommaSpaceWidth>,

    /// # Whether to align the equals sign in the key-value pairs.
    ///
    /// If `true`, the equals sign in the key-value pairs will be aligned.
    ///
    /// ⚠️ **WARNING** ⚠️\
    /// This feature does **not** apply to key-value pairs inside single line inline tables.
    ///
    /// ```toml
    /// # BEFORE
    /// key = "value1"
    /// key2 = "value2"
    /// key3.key4 = "value3"
    ///
    /// # AFTER
    /// key       = "value1"
    /// key2      = "value2"
    /// key3.key4 = "value3"
    /// ```
    #[cfg_attr(feature = "jsonschema", schemars(default = "bool::default"))]
    pub key_value_equals_sign_alignment: Option<bool>,

    /// # The preferred quote character for strings
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "StringQuoteStyle::default")
    )]
    pub string_quote_style: Option<StringQuoteStyle>,

    /// # Whether to align the trailing comments in the key-value pairs.
    ///
    /// If `true`, the trailing comments in value/key-value pairs will be aligned.
    ///
    /// **📝 NOTE 📝**\
    /// The trailing comments of table header are not targeted by alignment.
    ///
    /// ```toml
    /// # BEFORE
    /// key = "value1"  # comment 1
    /// key2 = "value2"  # comment 2
    /// key3.key4 = "value3"  # comment 3
    ///
    /// # AFTER
    /// key = "value1"        # comment 1
    /// key2 = "value2"       # comment 2
    /// key3.key4 = "value3"  # comment 3
    /// ```
    #[cfg_attr(feature = "jsonschema", schemars(default = "bool::default"))]
    pub trailing_comment_alignment: Option<bool>,

    /// # The number of spaces around the equals sign in a key-value pair.
    ///
    /// ```toml
    /// key = "value"
    /// #  ^ ^  <- this
    /// ```
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "KeyValueEqualsSignSpaceWidth::default")
    )]
    pub key_value_equals_sign_space_width: Option<KeyValueEqualsSignSpaceWidth>,

    /// # The type of line ending
    ///
    /// In TOML, the line ending must be either `LF` or `CRLF`.
    ///
    /// - `lf`: Line Feed only (`\n`), common on Linux and macOS as well as inside git repos.
    /// - `crlf`: Carriage Return Line Feed (`\r\n`), common on Windows.
    #[cfg_attr(feature = "jsonschema", schemars(default = "LineEnding::default"))]
    pub line_ending: Option<LineEnding>,

    /// # The maximum line width
    ///
    /// The formatter will try to keep lines within this width.
    #[cfg_attr(feature = "jsonschema", schemars(default = "LineWidth::default"))]
    pub line_width: Option<LineWidth>,

    /// # The number of spaces before the trailing comment.
    ///
    /// ```toml
    /// key = "value"  # trailing comment
    /// #            ^^  <- this
    /// ```
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "TrailingCommentSpaceWidth::default")
    )]
    pub trailing_comment_space_width: Option<TrailingCommentSpaceWidth>,
}

impl FormatRules {
    pub fn override_with(self, override_rules: &Self) -> Self {
        Self {
            array_bracket_space_width: self
                .array_bracket_space_width
                .or(override_rules.array_bracket_space_width),
            array_comma_space_width: self
                .array_comma_space_width
                .or(override_rules.array_comma_space_width),
            date_time_delimiter: self
                .date_time_delimiter
                .or(override_rules.date_time_delimiter),
            indent_style: self.indent_style.or(override_rules.indent_style),
            indent_sub_tables: self.indent_sub_tables.or(override_rules.indent_sub_tables),
            indent_table_key_value_pairs: self
                .indent_table_key_value_pairs
                .or(override_rules.indent_table_key_value_pairs),
            indent_width: self.indent_width.or(override_rules.indent_width),
            inline_table_brace_space_width: self
                .inline_table_brace_space_width
                .or(override_rules.inline_table_brace_space_width),
            inline_table_comma_space_width: self
                .inline_table_comma_space_width
                .or(override_rules.inline_table_comma_space_width),
            key_value_equals_sign_alignment: self
                .key_value_equals_sign_alignment
                .or(override_rules.key_value_equals_sign_alignment),
            string_quote_style: self
                .string_quote_style
                .or(override_rules.string_quote_style),
            trailing_comment_alignment: self
                .trailing_comment_alignment
                .or(override_rules.trailing_comment_alignment),
            key_value_equals_sign_space_width: self
                .key_value_equals_sign_space_width
                .or(override_rules.key_value_equals_sign_space_width),
            line_ending: self.line_ending.or(override_rules.line_ending),
            line_width: self.line_width.or(override_rules.line_width),
            trailing_comment_space_width: self
                .trailing_comment_space_width
                .or(override_rules.trailing_comment_space_width),
        }
    }
}
