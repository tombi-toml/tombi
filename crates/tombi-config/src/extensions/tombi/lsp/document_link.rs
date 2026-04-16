use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum TombiDocumentLinkFeatures {
    Enabled(EnabledOnly),
    Features(TombiDocumentLinkFeatureTree),
}

default_to_features!(TombiDocumentLinkFeatures, TombiDocumentLinkFeatureTree);

impl TombiDocumentLinkFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn path_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .path
                    .as_ref()
                    .map_or(true, ToggleFeatureDefaultTrue::enabled),
            }
    }
}

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
    pub path: Option<ToggleFeatureDefaultTrue>,
}
