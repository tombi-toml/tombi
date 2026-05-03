use crate::{BoolDefaultTrue, extensions::ToggleFeatureDefaultTrue};

use crate::extensions::EnabledOnly;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectReferencesFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectReferencesFeatureTree),
    Combined(PyprojectReferencesCombinedFeatureTree),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
pub struct PyprojectReferencesFeatureTree {
    /// # Dependency references feature
    ///
    /// Whether references list dependency-related usages.
    pub dependency: Option<ToggleFeatureDefaultTrue>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
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
pub struct PyprojectReferencesCombinedFeatureTree {
    /// # Enable references feature
    ///
    /// Whether to enable all `pyproject.toml` references features.
    pub enabled: Option<BoolDefaultTrue>,

    /// # Dependency references feature
    ///
    /// Whether references list dependency-related usages.
    pub dependency: Option<ToggleFeatureDefaultTrue>,
}

impl Default for PyprojectReferencesFeatures {
    fn default() -> Self {
        Self::Features(PyprojectReferencesFeatureTree::default())
    }
}

impl PyprojectReferencesFeatures {
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
            Self::Combined(features) => features.enabled.unwrap_or_default(),
        }
    }

    pub fn dependency(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.dependency,
            Self::Combined(features) => {
                if !features.enabled.unwrap_or_default().value() {
                    Some(ToggleFeatureDefaultTrue {
                        enabled: Some(false.into()),
                    })
                } else {
                    features.dependency.or(Some(Default::default()))
                }
            }
        }
    }
}
