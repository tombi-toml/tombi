/// FormatDefinitions provides the definition of the format that does not have the freedom set by [`FormatOptions`][crate::FormatOptions].
///
/// NOTE: Some of the items defined in FormatDefinitions may be moved to [`FormatOptions`][crate::FormatOptions] in the future.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Default, Clone, Copy)]
pub struct FormatDefinitions {}

impl FormatDefinitions {
    /// Returns the space before the trailing comment.
    ///
    /// ```toml
    /// key = "value"  # trailing comment
    /// #            ^^  <- this
    /// ```
    #[inline]
    pub const fn trailing_comment_space(&self) -> &'static str {
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
