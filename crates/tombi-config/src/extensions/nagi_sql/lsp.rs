use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum NagiSqlLspFeatures {
    Enabled(EnabledOnly),
    Features(NagiSqlLspFeatureTree),
}

toggle_features! {
    NagiSqlLspFeatures,

    #[derive(Debug, Default, Clone, PartialEq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
    #[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
    #[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
    #[cfg_attr(
        feature = "jsonschema",
        schemars(extend(
            "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
        ))
    )]
    pub struct NagiSqlLspFeatureTree {
        /// # Path completion feature
        pub completion: Option<ToggleFeatureDefaultTrue>,

        /// # Go-to-declaration feature
        pub goto_declaration: Option<ToggleFeatureDefaultTrue>,

        /// # Go-to-definition feature
        pub goto_definition: Option<ToggleFeatureDefaultTrue>,

        /// # Find-references feature
        pub references: Option<ToggleFeatureDefaultTrue>,
    }
}
