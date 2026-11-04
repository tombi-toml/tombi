use super::EnabledOnly;
use crate::{BoolDefaultTrue, SeverityLevel};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoLintFeatures {
    Enabled(EnabledOnly),
    Features(CargoLintFeatureTree),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, rename_all = "kebab-case")
)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CargoLintFeatureTree {
    /// # Cargo lint rules
    pub rules: Option<CargoLintRules>,
}

impl CargoLintFeatureTree {
    pub fn disabled() -> Self {
        Self {
            rules: Some(CargoLintRules::off()),
        }
    }
}

impl Default for CargoLintFeatures {
    fn default() -> Self {
        Self::Features(CargoLintFeatureTree::default())
    }
}

impl CargoLintFeatures {
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }
    pub fn feature_tree(&self) -> CargoLintFeatureTree {
        match self {
            Self::Enabled(enabled) if !enabled.enabled.value() => CargoLintFeatureTree::disabled(),
            Self::Enabled(_) => CargoLintFeatureTree::default(),
            Self::Features(features) => *features,
        }
    }
}

/// Severity levels for diagnostics produced by the Cargo extension.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, rename_all = "kebab-case", default)
)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CargoLintRules {
    /// Report an explicitly configured build script path that does not exist.
    pub missing_build_script: SeverityLevel,
    /// Report an explicitly configured package or workspace license file path that does not exist.
    pub missing_license_file: SeverityLevel,
    /// Report a path dependency whose directory does not contain Cargo.toml.
    pub missing_path_dependency: SeverityLevel,
    /// Report an explicitly configured package or workspace README path that does not exist.
    pub missing_readme: SeverityLevel,
    /// Report an explicitly configured target source path that does not exist.
    pub missing_target_path: SeverityLevel,
    /// Report a Cargo `workspace = true` reference without a matching workspace definition.
    pub missing_workspace: SeverityLevel,
    /// Report a `workspace.members` or `workspace.default-members` entry whose directory does not contain Cargo.toml, including every directory matched by a glob pattern.
    pub missing_workspace_member: SeverityLevel,
}

impl Default for CargoLintRules {
    fn default() -> Self {
        Self {
            missing_build_script: SeverityLevel::Error,
            missing_license_file: SeverityLevel::Error,
            missing_path_dependency: SeverityLevel::Error,
            missing_readme: SeverityLevel::Error,
            missing_target_path: SeverityLevel::Error,
            missing_workspace: SeverityLevel::Error,
            missing_workspace_member: SeverityLevel::Error,
        }
    }
}

impl CargoLintRules {
    pub fn off() -> Self {
        Self {
            missing_build_script: SeverityLevel::Off,
            missing_license_file: SeverityLevel::Off,
            missing_path_dependency: SeverityLevel::Off,
            missing_readme: SeverityLevel::Off,
            missing_target_path: SeverityLevel::Off,
            missing_workspace: SeverityLevel::Off,
            missing_workspace_member: SeverityLevel::Off,
        }
    }
}
