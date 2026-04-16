use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub enum CargoInlayHintFeatures {
    Enabled(EnabledOnly),
    Features(CargoInlayHintFeatureTree),
}

toggle_features! {
    CargoInlayHintFeatures,

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
    pub struct CargoInlayHintFeatureTree {
        /// # Dependency version inlay hint feature
        ///
        /// Whether inlay hints show dependency versions.
        pub dependency_version: Option<ToggleFeatureDefaultTrue>,

        /// # Default features inlay hint feature
        ///
        /// Whether inlay hints show `default-features` values.
        pub default_features: Option<ToggleFeatureDefaultTrue>,

        /// # Workspace value inlay hint feature
        ///
        /// Whether inlay hints show values inherited from the Cargo workspace.
        pub workspace_value: Option<ToggleFeatureDefaultTrue>,
    }
}
