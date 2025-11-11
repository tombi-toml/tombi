//! Formatting options
//!
//! Options for adjusting the formatting of TOML files.
//! Initially, this structure contained settings related to `line-width`, etc.,
//! but to avoid unnecessary discussions about the format, all settings have been moved to [formatter::FormatDefinition].
//! In the future, there is a possibility that options will be added to this structure,
//! but considering the recent trend of formatters to avoid such discussions by restricting the settings and its results,
//! this structure is currently empty.

use crate::{
    ArrayBracketSpaceWidth, ArrayElementSpaceWidth, DateTimeDelimiter, IndentStyle, IndentWidth,
    InlineTableBraceSpaceWidth, InlineTableElementSpaceWidth, LineEnding, LineWidth, QuoteStyle,
    TrailingCommentSpaceWidth,
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
        schemars(default = "ArrayElementSpaceWidth::default")
    )]
    pub array_element_space_width: Option<ArrayElementSpaceWidth>,

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
        schemars(default = "InlineTableElementSpaceWidth::default")
    )]
    pub inline_table_element_space_width: Option<InlineTableElementSpaceWidth>,

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
