use tombi_config::{DateTimeDelimiter, IndentStyle, IndentWidth, LineEnding, LineWidth, QuoteStyle};

/// FormatDefinitions provides the definition of the format that does not have the freedom set by [`FormatOptions`][crate::FormatOptions].
///
/// NOTE: Some of the items defined in FormatDefinitions may be moved to [`FormatOptions`][crate::FormatOptions] in the future.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy)]
pub struct FormatDefinitions {
    /// # The style of indentation.
    ///
    /// Whether to use spaces or tabs for indentation.
    #[cfg_attr(feature = "jsonschema", schemars(default = "IndentStyle::default"))]
    pub indent_style: Option<IndentStyle>,

    /// # The number of spaces per indentation level.
    #[cfg_attr(feature = "jsonschema", schemars(default = "IndentWidth::default"))]
    pub indent_width: Option<IndentWidth>,

    /// # The maximum line width.
    ///
    /// The formatter will try to keep lines within this width.
    #[cfg_attr(feature = "jsonschema", schemars(default = "LineWidth::default"))]
    pub line_width: Option<LineWidth>,

    /// # The type of line ending.
    ///
    /// In TOML, the line ending must be either `LF` or `CRLF`.
    ///
    /// - `lf`: Line Feed only (`\n`), common on Linux and macOS as well as inside git repos.
    /// - `crlf`: Carriage Return Line Feed (`\r\n`), common on Windows.
    #[cfg_attr(feature = "jsonschema", schemars(default = "LineEnding::default"))]
    pub line_ending: Option<LineEnding>,

    /// # The delimiter between date and time.
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

    /// # The preferred quote character for strings.
    pub quote_style: Option<QuoteStyle>,
}

impl FormatDefinitions {
    pub const fn default() -> Self {
        Self {
            indent_style: None,
            indent_width: None,
            line_width: None,
            line_ending: None,
            date_time_delimiter: None,
            quote_style: None,
        }
    }

    /// Returns the space before the tailing comment.
    ///
    /// ```toml
    /// key = "value"  # tailing comment
    /// #            ^^  <- this
    /// ```
    #[inline]
    pub const fn tailing_comment_space(&self) -> &'static str {
        "  "
    }

    /// Returns the space inside the brackets of an array.
    ///
    /// ```toml
    /// key = [ 1, 2, 3 ]
    /// #      ^       ^  <- this
    #[inline]
    pub const fn singleline_array_bracket_inner_space(&self) -> &'static str {
        ""
    }

    /// Returns the space after the comma in an array.
    ///
    /// ```toml
    /// key = [ 1, 2, 3 ]
    /// #         ^  ^    <- this
    #[inline]
    pub const fn singleline_array_space_after_comma(&self) -> &'static str {
        " "
    }

    /// Returns the space inside the brackets of an inline table.
    ///
    /// ```toml
    /// key = { a = 1, b = 2 }
    /// #      ^            ^  <- this
    /// ```
    #[inline]
    pub const fn singleline_inline_table_brace_inner_space(&self) -> &'static str {
        " "
    }

    /// Returns the space after the comma in an inline table.
    ///
    /// ```toml
    /// key = { a = 1, b = 2 }
    /// #             ^  <- this
    /// ```
    #[inline]
    pub const fn singleline_inline_table_space_after_comma(&self) -> &'static str {
        " "
    }
}

impl Default for FormatDefinitions {
    fn default() -> Self {
        Self::default()
    }
}
