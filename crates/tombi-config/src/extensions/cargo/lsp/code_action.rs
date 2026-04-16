use crate::{
    BoolDefaultTrue,
    extensions::{EnabledOnly, ToggleFeatureDefaultTrue},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoCodeActionFeatures {
    Enabled(EnabledOnly),
    Features(CargoCodeActionFeatureTree),
}

default_to_features!(CargoCodeActionFeatures, CargoCodeActionFeatureTree);

impl CargoCodeActionFeatures {
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }

    pub fn inherit_from_workspace(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.inherit_from_workspace,
        }
    }

    pub fn inherit_dependency_from_workspace(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.inherit_dependency_from_workspace,
        }
    }

    pub fn convert_dependency_to_table_format(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.convert_dependency_to_table_format,
        }
    }

    pub fn add_to_workspace_and_inherit_dependency(&self) -> Option<ToggleFeatureDefaultTrue> {
        match self {
            Self::Enabled(enabled) => enabled.into(),
            Self::Features(features) => features.add_to_workspace_and_inherit_dependency,
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
pub struct CargoCodeActionFeatureTree {
    /// # Inherit-from-workspace code action feature
    ///
    /// Whether code actions can replace a value with `workspace = true`.
    pub inherit_from_workspace: Option<ToggleFeatureDefaultTrue>,

    /// # Inherit-dependency-from-workspace code action feature
    ///
    /// Whether code actions can inherit dependency settings from the workspace.
    pub inherit_dependency_from_workspace: Option<ToggleFeatureDefaultTrue>,

    /// # Convert-dependency-to-table-format code action feature
    ///
    /// Whether code actions can rewrite inline dependencies to table format.
    pub convert_dependency_to_table_format: Option<ToggleFeatureDefaultTrue>,

    /// # Add-to-workspace-and-inherit-dependency code action feature
    ///
    /// Whether code actions can add a dependency to the workspace and inherit it.
    pub add_to_workspace_and_inherit_dependency: Option<ToggleFeatureDefaultTrue>,
}
