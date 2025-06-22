#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Schema)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LintOptions {
    /// # Lint rules.
    pub rules: Option<LintRules>,
}

impl LintOptions {
    pub const fn default() -> Self {
        Self { rules: None }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::VersionSort)))]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct LintRules {
    /// # Key empty.
    ///
    /// Check if the key is empty.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// "" = true
    /// ```
    pub key_empty: Option<SeverityLevelDefaultWarn>,

    /// # Dotted keys out of order.
    ///
    /// Check if dotted keys are defined out of order.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// apple.type = "fruit"
    /// orange.type = "fruit"
    /// apple.skin = "thin"
    /// orange.skin = "thick"
    ///
    /// # RECOMMENDED
    /// apple.type = "fruit"
    /// apple.skin = "thin"
    /// orange.type = "fruit"
    /// orange.skin = "thick"
    /// ```
    pub dotted_keys_out_of_order: Option<SeverityLevelDefaultWarn>,

    /// # Tables out of order.
    ///
    /// Check if tables are defined out of order.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// [fruit.apple]
    /// [animal]
    /// [fruit.orange]
    ///
    /// # RECOMMENDED
    /// [fruit.apple]
    /// [fruit.orange]
    /// [animal]
    /// ```
    pub tables_out_of_order: Option<SeverityLevelDefaultWarn>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum SeverityLevel {
    /// # Disable the Rule.
    Off,

    /// # Display as Warning.
    Warn,

    /// # Display as Error.
    Error,
}

macro_rules! severity_level_wrapper {
    ($name:ident, $level:ident, $default:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
        #[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
        #[cfg_attr(feature = "jsonschema", schemars(extend("default" = $default)))]
        pub struct $name(SeverityLevel);

        impl Default for $name {
            fn default() -> Self {
                Self(SeverityLevel::$level)
            }
        }

        impl From<$name> for SeverityLevel {
            fn from(level: $name) -> Self {
                level.0
            }
        }
    };
}

severity_level_wrapper!(SeverityLevelDefaultWarn, Warn, "warn");
severity_level_wrapper!(SeverityLevelDefaultError, Error, "error");
severity_level_wrapper!(SeverityLevelDefaultOff, Off, "off");
