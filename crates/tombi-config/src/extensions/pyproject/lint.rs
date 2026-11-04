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
    ///
    /// ```toml
    /// [project]
    /// import-names = ["acme"] # conflicts with the same name declared below
    /// import-namespaces = ["acme"] # conflicts with the same name declared above
    /// ```
    pub ambiguous_import_name: SeverityLevel,
    /// Diagnose cyclic dependency group includes.
    ///
    /// ```toml
    /// [dependency-groups]
    /// dev = [{ include-group = "test" }] # includes test
    /// test = [{ include-group = "dev" }] # includes dev, creating a cycle
    /// ```
    pub cyclic_include_group: SeverityLevel,
    /// Diagnose dependency group names that collide after PEP 503 normalization.
    ///
    /// ```toml
    /// [dependency-groups]
    /// qa_tools = [] # normalizes to "qa-tools", same as the name below
    /// "QA.Tools" = [] # normalizes to "qa-tools", same as the name above
    /// ```
    pub duplicate_dependency_group: SeverityLevel,
    /// Diagnose project extra names that collide after PEP 503 normalization.
    ///
    /// ```toml
    /// [project.optional-dependencies]
    /// qa_tools = [] # normalizes to "qa-tools", same as the name below
    /// "QA.Tools" = [] # normalizes to "qa-tools", same as the name above
    /// ```
    pub duplicate_extra_name: SeverityLevel,
    /// Diagnose duplicate Python import names.
    ///
    /// ```toml
    /// [project]
    /// import-names = ["acme", "acme"] # the same import name is declared twice
    /// ```
    pub duplicate_import_name: SeverityLevel,
    /// Diagnose invalid PEP 508 dependency specifiers.
    ///
    /// ```toml
    /// [project]
    /// dependencies = ["not a valid requirement !!!"] # not a valid PEP 508 requirement
    /// ```
    pub invalid_dependency_specifier: SeverityLevel,
    /// Diagnose invalid author and maintainer email display names.
    ///
    /// ```toml
    /// [project]
    /// authors = [{ name = "Doe, Jane" }] # commas are not allowed in the display name
    /// ```
    pub invalid_email_display_name: SeverityLevel,
    /// Diagnose invalid Python entry point targets.
    ///
    /// ```toml
    /// [project.scripts]
    /// cli = "my_package.class" # `class` is a Python keyword and cannot be an attribute
    /// ```
    pub invalid_entry_point_target: SeverityLevel,
    /// Diagnose invalid Python import names.
    ///
    /// ```toml
    /// [project]
    /// import-names = ["if"] # `if` is a Python keyword, not a valid import name
    /// ```
    pub invalid_import_name: SeverityLevel,
    /// Diagnose import names whose parent is not declared.
    ///
    /// ```toml
    /// [project]
    /// import-namespaces = ["acme.plugins"] # parent `acme` is not declared in either list
    /// ```
    pub missing_import_name_parent: SeverityLevel,
    /// Diagnose a non-glob `tool.uv.workspace.members` path whose directory does not contain pyproject.toml.
    ///
    /// ```toml
    /// [tool.uv.workspace]
    /// members = ["packages/missing"] # this directory has no pyproject.toml
    /// ```
    pub missing_workspace_member: SeverityLevel,
    /// Diagnose dependency groups that include an undefined group.
    ///
    /// ```toml
    /// [dependency-groups]
    /// dev = [{ include-group = "missing" }] # no [dependency-groups].missing exists
    /// ```
    pub undefined_include_group: SeverityLevel,
    /// Diagnose classifiers not listed in the Trove classifier specification.
    ///
    /// ```toml
    /// [project]
    /// classifiers = ["NotAClassifier :: DefinitelyInvalid"] # not in the Trove classifier list
    /// ```
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
