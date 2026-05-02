use crate::extensions::{EnabledOnly, ToggleFeatureDefaultTrue};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectGotoDeclarationFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectGotoDeclarationFeatureTree),
}

toggle_features! {
    PyprojectGotoDeclarationFeatures,

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
    pub struct PyprojectGotoDeclarationFeatureTree {
        /// # Dependency declaration navigation feature
        ///
        /// Whether declaration navigation resolves dependency declarations.
        pub dependency: Option<ToggleFeatureDefaultTrue>,

        /// # Member declaration navigation feature
        ///
        /// Whether declaration navigation resolves workspace member declarations.
        pub member: Option<ToggleFeatureDefaultTrue>,

        /// # Deprecated path declaration navigation feature
        ///
        /// Deprecated. This setting is accepted for backward compatibility but ignored.
        #[cfg_attr(feature = "jsonschema", schemars(extend("deprecated" = true)))]
        pub path: Option<ToggleFeatureDefaultTrue>,
    }
}
