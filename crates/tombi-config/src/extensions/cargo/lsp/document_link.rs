use crate::{
    ToggleFeatureDefaultFalse,
    extensions::{EnabledOnly, ToggleFeatureDefaultTrue},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoDocumentLinkFeatures {
    Enabled(EnabledOnly),
    Features(CargoDocumentLinkFeatureTree),
}

toggle_features! {
    CargoDocumentLinkFeatures,

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
    pub struct CargoDocumentLinkFeatureTree {
        /// # Deprecated Cargo.toml document link feature
        ///
        /// Deprecated. This setting is accepted for backward compatibility and will be removed in a future release.
        #[cfg_attr(feature = "jsonschema", schemars(extend("deprecated" = true)))]
        pub cargo_toml: Option<ToggleFeatureDefaultFalse>,

        /// # crates.io document link feature
        ///
        /// Whether document links are created for crates.io package references.
        pub crates_io: Option<ToggleFeatureDefaultTrue>,

        /// # Deprecated Git document link feature
        ///
        /// Deprecated. This setting is accepted for backward compatibility and will be removed in a future release.
        #[cfg_attr(feature = "jsonschema", schemars(extend("deprecated" = true)))]
        pub git: Option<ToggleFeatureDefaultFalse>,

        /// # Deprecated path document link feature
        ///
        /// Deprecated. This setting is accepted for backward compatibility and will be removed in a future release.
        #[cfg_attr(feature = "jsonschema", schemars(extend("deprecated" = true)))]
        pub path: Option<ToggleFeatureDefaultFalse>,

        /// # Deprecated workspace document link feature
        ///
        /// Deprecated. This setting is accepted for backward compatibility and will be removed in a future release.
        #[cfg_attr(feature = "jsonschema", schemars(extend("deprecated" = true)))]
        pub workspace: Option<ToggleFeatureDefaultFalse>,
    }
}
