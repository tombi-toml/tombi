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
        /// # Cargo.toml document link feature
        ///
        /// Whether document links are created for `Cargo.toml` references.
        pub cargo_toml: Option<ToggleFeatureDefaultFalse>,

        /// # crates.io document link feature
        ///
        /// Whether document links are created for crates.io package references.
        pub crates_io: Option<ToggleFeatureDefaultTrue>,

        /// # Git document link feature
        ///
        /// Whether document links are created for Git references.
        pub git: Option<ToggleFeatureDefaultFalse>,

        /// # Path document link feature
        ///
        /// Whether document links are created for filesystem paths.
        pub path: Option<ToggleFeatureDefaultFalse>,

        /// # Workspace document link feature
        ///
        /// Whether document links are created for `workspace = true` references.
        pub workspace: Option<ToggleFeatureDefaultFalse>,
    }
}
