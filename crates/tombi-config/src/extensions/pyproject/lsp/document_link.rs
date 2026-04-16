use crate::{
    ToggleFeatureDefaultFalse,
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

toggle_features! {
    PyprojectDocumentLinkFeatures,

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
        pub pyproject_toml: Option<ToggleFeatureDefaultFalse>,

        /// # PyPI document link feature
        ///
        /// Whether document links are created for `pypi.org` package references.
        pub pypi_org: Option<ToggleFeatureDefaultTrue>,
    }
}
