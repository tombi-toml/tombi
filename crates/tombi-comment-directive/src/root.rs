use tombi_toml_version::TomlVersion;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/root-tombi-directive.json")))]
pub struct RootCommentDirective {
    /// # TOML version.
    ///
    /// This directive specifies the TOML version of this document, with the highest priority.
    pub toml_version: Option<TomlVersion>,
}
