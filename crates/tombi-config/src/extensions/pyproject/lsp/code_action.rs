use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectCodeActionFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectCodeActionFeatureTree),
}

toggle_features! {
    PyprojectCodeActionFeatures,

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
    pub struct PyprojectCodeActionFeatureTree {
        /// # Use-workspace-dependency code action feature
        ///
        /// Whether code actions can reuse a dependency declared in the workspace.
        pub use_workspace_dependency: Option<ToggleFeatureDefaultTrue>,

        /// # Add-to-workspace-and-use-workspace-dependency code action feature
        ///
        /// Whether code actions can add a dependency to the workspace and reuse it.
        pub add_to_workspace_and_use_workspace_dependency: Option<ToggleFeatureDefaultTrue>,
    }
}
