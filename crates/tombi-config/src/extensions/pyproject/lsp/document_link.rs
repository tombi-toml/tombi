use crate::{
    BoolDefaultTrue,
    extensions::{EnabledOnly, ToggleFeatureDefaultTrue},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectDocumentLinkFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectDocumentLinkFeatureTree),
}

default_to_features!(
    PyprojectDocumentLinkFeatures,
    PyprojectDocumentLinkFeatureTree
);

impl PyprojectDocumentLinkFeatures {
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }

    pub fn pyproject_toml(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.pyproject_toml,
        }
    }

    pub fn pypi_org(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.pypi_org,
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
pub struct PyprojectDocumentLinkFeatureTree {
    /// # pyproject.toml document link feature
    ///
    /// Whether document links are created for `pyproject.toml` references.
    pub pyproject_toml: Option<ToggleFeatureDefaultTrue>,

    /// # PyPI document link feature
    ///
    /// Whether document links are created for `pypi.org` package references.
    pub pypi_org: Option<ToggleFeatureDefaultTrue>,
}
