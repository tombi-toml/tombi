use crate::{ToggleFeatureDefaultFalse, extensions::EnabledOnly};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum TombiDocumentLinkFeatures {
    Enabled(EnabledOnly),
    Features(TombiDocumentLinkFeatureTree),
}

toggle_features! {
    TombiDocumentLinkFeatures,

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
    pub struct TombiDocumentLinkFeatureTree {
        /// # Path document link feature
        ///
        /// Whether document links are created for filesystem paths.
        pub path: Option<ToggleFeatureDefaultFalse>,
    }
}
