use crate::{
    BoolDefaultTrue,
    extensions::{EnabledOnly, ToggleFeatureDefaultTrue},
};

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
pub enum PyprojectHoverFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectHoverFeatureTree),
}

default_to_features!(PyprojectHoverFeatures, PyprojectHoverFeatureTree);

impl PyprojectHoverFeatures {
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }

    pub fn dependency_detail(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.dependency_detail,
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
pub struct PyprojectHoverFeatureTree {
    /// # Dependency detail hover feature
    ///
    /// Whether hover shows detailed dependency metadata.
    pub dependency_detail: Option<ToggleFeatureDefaultTrue>,
}
