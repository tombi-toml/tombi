use crate::{
    BoolDefaultTrue,
    extensions::{EnabledOnly, ToggleFeatureDefaultTrue},
};

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
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }

    pub fn dependency_version(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.dependency_version,
        }
    }

    pub fn dependency_feature(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.dependency_feature,
        }
    }

    pub fn path(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.path,
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
