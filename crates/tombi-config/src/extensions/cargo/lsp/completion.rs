use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoCompletionFeatures {
    Enabled(EnabledOnly),
    Features(CargoCompletionFeatureTree),
}

toggle_features! {
    CargoCompletionFeatures,

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
    pub struct CargoCompletionFeatureTree {
        /// # Dependency version completion feature
        ///
        /// Whether completion suggests dependency versions.
        pub dependency_version: Option<ToggleFeatureDefaultTrue>,

        /// # Dependency feature completion feature
        ///
        /// Whether completion suggests dependency features.
        pub dependency_feature: Option<ToggleFeatureDefaultTrue>,

        /// # Path completion feature
        ///
        /// Whether completion suggests filesystem paths.
        pub path: Option<ToggleFeatureDefaultTrue>,
    }
}
