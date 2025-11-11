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
    InlineTableBraceSpaceWidth, InlineTableCommaSpaceWidth, KeyValueEqualSpaceWidth, LineEnding,
    LineWidth, QuoteStyle, TrailingCommentSpaceWidth,
};

/// # Formatter options
///
/// To avoid needless discussion of formatting rules,
/// we do not currently have a configuration item for formatting.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FormatOptions {
    /// # Format rules
    pub rules: Option<FormatRules>,

    /// # The number of spaces inside the brackets of a single line array.
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.array-bracket-space-width` instead.
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "ArrayBracketSpaceWidth::default")
    )]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub array_bracket_space_width: Option<ArrayBracketSpaceWidth>,

    /// # The number of spaces after the comma in a single line array.
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.array-comma-space-width` instead.
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "ArrayCommaSpaceWidth::default")
    )]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub array_element_space_width: Option<ArrayCommaSpaceWidth>,

    /// # The delimiter between date and time
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.date-time-delimiter` instead.
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "DateTimeDelimiter::default")
    )]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub date_time_delimiter: Option<DateTimeDelimiter>,

    /// # The style of indentation
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.indent-style` instead.
    #[cfg_attr(feature = "jsonschema", schemars(default = "IndentStyle::default"))]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub indent_style: Option<IndentStyle>,

    /// # The number of spaces per indentation level
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.indent-width` instead.
    #[cfg_attr(feature = "jsonschema", schemars(default = "IndentWidth::default"))]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub indent_width: Option<IndentWidth>,

    /// # The number of spaces inside the brackets of a single line inline table.
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.inline-table-brace-space-width` instead.
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "InlineTableBraceSpaceWidth::default")
    )]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub inline_table_brace_space_width: Option<InlineTableBraceSpaceWidth>,

    /// # The number of spaces after the comma in a single line inline table.
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.inline-table-comma-space-width` instead.
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "InlineTableCommaSpaceWidth::default")
    )]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub inline_table_element_space_width: Option<InlineTableCommaSpaceWidth>,

    /// # The type of line ending
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.line-ending` instead.
    #[cfg_attr(feature = "jsonschema", schemars(default = "LineEnding::default"))]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub line_ending: Option<LineEnding>,

    /// # The maximum line width
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.line-width` instead.
    #[cfg_attr(feature = "jsonschema", schemars(default = "LineWidth::default"))]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub line_width: Option<LineWidth>,

    /// # The preferred quote character for strings
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.quote-style` instead.
    #[cfg_attr(feature = "jsonschema", schemars(default = "QuoteStyle::default"))]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub quote_style: Option<QuoteStyle>,

    /// # The number of spaces before the trailing comment.
    ///
    /// **🚧 Deprecated 🚧**\
    /// Please use `format.rules.trailing-comment-space-width` instead.
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "TrailingCommentSpaceWidth::default")
    )]
    #[cfg_attr(feature = "jsonschema", deprecated)]
    pub trailing_comment_space_width: Option<TrailingCommentSpaceWidth>,
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
    /// - `T`: Example: `2001-01-01T00:00:00`
    /// - `space`: Example: `2001-01-01 00:00:00`
    /// - `preserve`: Preserve the original delimiter.
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "DateTimeDelimiter::default")
    )]
    pub date_time_delimiter: Option<DateTimeDelimiter>,

    /// # The style of indentation
    ///
    /// Whether to use spaces or tabs for indentation.
    #[cfg_attr(feature = "jsonschema", schemars(default = "IndentStyle::default"))]
    pub indent_style: Option<IndentStyle>,

    /// # The number of spaces per indentation level
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

    /// # The number of spaces after the equal in a key-value pair.
    ///
    /// ```toml
    /// key = "value"
    /// #  ^ ^  <- this
    /// ```
    #[cfg_attr(
        feature = "jsonschema",
        schemars(default = "KeyValueEqualSpaceWidth::default")
    )]
    pub key_value_equal_space_width: Option<KeyValueEqualSpaceWidth>,

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

    /// # The preferred quote character for strings
    #[cfg_attr(feature = "jsonschema", schemars(default = "QuoteStyle::default"))]
    pub quote_style: Option<QuoteStyle>,

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
