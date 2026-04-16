use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoNavigationFeatures {
    Enabled(EnabledOnly),
    Features(CargoNavigationFeatureTree),
}

toggle_features! {
    CargoNavigationFeatures,

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
    pub struct CargoNavigationFeatureTree {
        /// # Dependency navigation feature
        ///
        /// Whether navigation resolves dependency definitions and declarations.
        pub dependency: Option<ToggleFeatureDefaultTrue>,

        /// # Member navigation feature
        ///
        /// Whether navigation resolves workspace member targets.
        pub member: Option<ToggleFeatureDefaultTrue>,

        /// # Path navigation feature
        ///
        /// Whether navigation resolves filesystem paths.
        pub path: Option<ToggleFeatureDefaultTrue>,
    }
}
