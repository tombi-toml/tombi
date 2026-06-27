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
pub enum CargoHoverFeatures {
    Enabled(EnabledOnly),
    Features(CargoHoverFeatureTree),
}

toggle_features! {
    CargoHoverFeatures,

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
    pub struct CargoHoverFeatureTree {
        /// # Dependency detail hover feature
        ///
        /// Whether hover shows detailed dependency metadata.
        pub dependency_detail: Option<ToggleFeatureDefaultTrue>,

        /// # Default features hover feature
        ///
        /// Whether hover shows default Cargo dependency features.
        pub default_features: Option<ToggleFeatureDefaultTrue>,

        /// # Feature dependencies hover feature
        ///
        /// Whether hover shows dependencies of the selected Cargo feature.
        pub feature_dependencies: Option<ToggleFeatureDefaultTrue>,
    }
}
