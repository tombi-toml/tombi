use super::EnabledOnly;
use crate::{BoolDefaultTrue, SeverityLevel};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectLintFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectLintFeatureTree),
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
pub struct PyprojectLintFeatureTree {
    /// # Pyproject lint rules
    pub rules: Option<PyprojectLintRules>,
}

impl PyprojectLintFeatureTree {
    pub fn disabled() -> Self {
        Self {
            rules: Some(PyprojectLintRules::off()),
        }
    }
}

impl Default for PyprojectLintFeatures {
    fn default() -> Self {
        Self::Features(PyprojectLintFeatureTree::default())
    }
}

impl PyprojectLintFeatures {
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }

    pub fn rules(&self) -> Option<&PyprojectLintRules> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.rules.as_ref(),
        }
    }

    pub fn feature_tree(&self) -> PyprojectLintFeatureTree {
        match self {
            Self::Enabled(enabled) if !enabled.enabled.value() => {
                PyprojectLintFeatureTree::disabled()
            }
            Self::Enabled(_) => PyprojectLintFeatureTree::default(),
            Self::Features(features) => *features,
        }
    }
}

/// Severity levels for diagnostics produced by the pyproject extension.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(deny_unknown_fields, rename_all = "kebab-case")
)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", serde(default))]
pub struct PyprojectLintRules {
    /// Diagnose names declared as both exclusive and shared imports.
    pub ambiguous_import_name: SeverityLevel,
    /// Diagnose cyclic dependency group includes.
    pub cyclic_include_group: SeverityLevel,
    /// Diagnose dependency group names that collide after normalization.
    pub duplicate_dependency_group: SeverityLevel,
    /// Diagnose project extra names that collide after normalization.
    pub duplicate_extra_name: SeverityLevel,
    /// Diagnose duplicate Python import names.
    pub duplicate_import_name: SeverityLevel,
    /// Diagnose invalid PEP 508 dependency specifiers.
    pub invalid_dependency_specifier: SeverityLevel,
    /// Diagnose invalid author and maintainer email display names.
    pub invalid_email_display_name: SeverityLevel,
    /// Diagnose invalid Python entry point targets.
    pub invalid_entry_point_target: SeverityLevel,
    /// Diagnose invalid Python import names.
    pub invalid_import_name: SeverityLevel,
    /// Diagnose import names whose parent is not declared.
    pub missing_import_name_parent: SeverityLevel,
    /// Diagnose a non-glob `tool.uv.workspace.members` path whose directory does not contain pyproject.toml.
    pub missing_workspace_member: SeverityLevel,
    /// Diagnose dependency groups that include an undefined group.
    pub undefined_include_group: SeverityLevel,
    /// Diagnose classifiers not listed in the Trove classifier specification.
    pub unknown_classifier: SeverityLevel,
}

impl Default for PyprojectLintRules {
    fn default() -> Self {
        Self {
            ambiguous_import_name: SeverityLevel::Error,
            cyclic_include_group: SeverityLevel::Error,
            duplicate_dependency_group: SeverityLevel::Error,
            duplicate_extra_name: SeverityLevel::Error,
            duplicate_import_name: SeverityLevel::Warn,
            invalid_dependency_specifier: SeverityLevel::Error,
            invalid_email_display_name: SeverityLevel::Error,
            invalid_entry_point_target: SeverityLevel::Error,
            invalid_import_name: SeverityLevel::Error,
            missing_import_name_parent: SeverityLevel::Warn,
            missing_workspace_member: SeverityLevel::Error,
            undefined_include_group: SeverityLevel::Error,
            unknown_classifier: SeverityLevel::Warn,
        }
    }
}

impl PyprojectLintRules {
    pub fn off() -> Self {
        Self {
            ambiguous_import_name: SeverityLevel::Off,
            cyclic_include_group: SeverityLevel::Off,
            duplicate_dependency_group: SeverityLevel::Off,
            duplicate_extra_name: SeverityLevel::Off,
            duplicate_import_name: SeverityLevel::Off,
            invalid_dependency_specifier: SeverityLevel::Off,
            invalid_email_display_name: SeverityLevel::Off,
            invalid_entry_point_target: SeverityLevel::Off,
            invalid_import_name: SeverityLevel::Off,
            missing_import_name_parent: SeverityLevel::Off,
            missing_workspace_member: SeverityLevel::Off,
            undefined_include_group: SeverityLevel::Off,
            unknown_classifier: SeverityLevel::Off,
        }
    }
}
