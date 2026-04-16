use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoCompletionFeatures {
    Enabled(EnabledOnly),
    Features(CargoCompletionFeatureTree),
}

default_to_features!(CargoCompletionFeatures, CargoCompletionFeatureTree);

impl CargoCompletionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn dependency_version_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency_version
                    .as_ref()
                    .map_or(true, ToggleFeatureDefaultTrue::enabled),
            }
    }

    pub fn dependency_feature_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency_feature
                    .as_ref()
                    .map_or(true, ToggleFeatureDefaultTrue::enabled),
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
