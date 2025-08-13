use tombi_toml_version::TomlVersion;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/tombi-directive.json")))]
pub struct TombiDirective {
    /// # TOML version.
    ///
    /// This directive specifies the TOML version of this document, with the highest priority.
    pub toml_version: Option<TomlVersion>,
}
