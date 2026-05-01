use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectGotoDefinitionFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectGotoDefinitionFeatureTree),
}

toggle_features! {
    PyprojectGotoDefinitionFeatures,

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
    pub struct PyprojectGotoDefinitionFeatureTree {
        /// # Dependency definition navigation feature
        ///
        /// Whether definition navigation resolves dependency targets.
        pub dependency: Option<ToggleFeatureDefaultTrue>,

        /// # Member definition navigation feature
        ///
        /// Whether definition navigation resolves workspace member targets.
        pub member: Option<ToggleFeatureDefaultTrue>,

        /// # Path definition navigation feature
        ///
        /// Whether definition navigation resolves filesystem paths.
        pub path: Option<ToggleFeatureDefaultTrue>,
    }
}
