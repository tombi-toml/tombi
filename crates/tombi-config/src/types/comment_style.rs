/// The style used to format comments.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CommentStyle {
    /// Normalize comment text according to Tombi's formatting rules.
    #[default]
    Normalize,

    /// Preserve the original comment text while formatting its placement normally.
    Preserve,
}
