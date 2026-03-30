use super::{EnabledOnly, ToggleFeature, default_to_features};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectExtensionFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectExtensionFeatureTree),
}

default_to_features!(PyprojectExtensionFeatures, PyprojectExtensionFeatureTree);

impl PyprojectExtensionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn lsp(&self) -> Option<&PyprojectLspFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.lsp.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::completion_enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::path_completion_enabled)
    }

    pub fn inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::inlay_hint_enabled)
    }

    pub fn dependency_version_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                PyprojectLspFeatures::dependency_version_inlay_hint_enabled,
            )
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::goto_definition_enabled)
    }

    pub fn goto_definition_dependency_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                PyprojectLspFeatures::goto_definition_dependency_enabled,
            )
    }

    pub fn goto_definition_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::goto_definition_member_enabled)
    }

    pub fn goto_definition_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::goto_definition_path_enabled)
    }

    pub fn goto_declaration_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::goto_declaration_enabled)
    }

    pub fn goto_declaration_dependency_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                PyprojectLspFeatures::goto_declaration_dependency_enabled,
            )
    }

    pub fn goto_declaration_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::goto_declaration_member_enabled)
    }

    pub fn goto_declaration_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::goto_declaration_path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::document_link_enabled)
    }

    pub fn pyproject_toml_document_link_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                PyprojectLspFeatures::pyproject_toml_document_link_enabled,
            )
    }

    pub fn pypi_org_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::pypi_org_document_link_enabled)
    }

    pub fn hover_enabled(&self) -> bool {
        self.enabled() && self.lsp().map_or(true, PyprojectLspFeatures::hover_enabled)
    }

    pub fn dependency_detail_hover_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::dependency_detail_hover_enabled)
    }

    pub fn code_action_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, PyprojectLspFeatures::code_action_enabled)
    }

    pub fn use_workspace_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                PyprojectLspFeatures::use_workspace_dependency_code_action_enabled,
            )
    }

    pub fn add_to_workspace_and_use_workspace_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                PyprojectLspFeatures::add_to_workspace_and_use_workspace_dependency_code_action_enabled,
            )
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
pub struct PyprojectExtensionFeatureTree {
    /// # Pyproject LSP feature options
    pub lsp: Option<PyprojectLspFeatures>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectLspFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectLspFeatureTree),
}

default_to_features!(PyprojectLspFeatures, PyprojectLspFeatureTree);

impl PyprojectLspFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn completion(&self) -> Option<&PyprojectCompletionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.completion.as_ref(),
        }
    }

    pub fn inlay_hint(&self) -> Option<&PyprojectInlayHintFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.inlay_hint.as_ref(),
        }
    }

    pub fn goto_definition(&self) -> Option<&PyprojectNavigationFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_definition.as_ref(),
        }
    }

    pub fn goto_declaration(&self) -> Option<&PyprojectNavigationFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_declaration.as_ref(),
        }
    }

    pub fn document_link(&self) -> Option<&PyprojectDocumentLinkFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.document_link.as_ref(),
        }
    }

    pub fn code_action(&self) -> Option<&PyprojectCodeActionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.code_action.as_ref(),
        }
    }

    pub fn hover(&self) -> Option<&PyprojectHoverFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.hover.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, PyprojectCompletionFeatures::enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, PyprojectCompletionFeatures::path_enabled)
    }

    pub fn inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, PyprojectInlayHintFeatures::enabled)
    }

    pub fn dependency_version_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, PyprojectInlayHintFeatures::dependency_version_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, PyprojectNavigationFeatures::enabled)
    }

    pub fn goto_definition_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, PyprojectNavigationFeatures::dependency_enabled)
    }

    pub fn goto_definition_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, PyprojectNavigationFeatures::member_enabled)
    }

    pub fn goto_definition_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, PyprojectNavigationFeatures::path_enabled)
    }

    pub fn goto_declaration_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, PyprojectNavigationFeatures::enabled)
    }

    pub fn goto_declaration_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, PyprojectNavigationFeatures::dependency_enabled)
    }

    pub fn goto_declaration_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, PyprojectNavigationFeatures::member_enabled)
    }

    pub fn goto_declaration_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, PyprojectNavigationFeatures::path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, PyprojectDocumentLinkFeatures::enabled)
    }

    pub fn pyproject_toml_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, PyprojectDocumentLinkFeatures::pyproject_toml_enabled)
    }

    pub fn pypi_org_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, PyprojectDocumentLinkFeatures::pypi_org_enabled)
    }

    pub fn hover_enabled(&self) -> bool {
        self.enabled() && self.hover().map_or(true, PyprojectHoverFeatures::enabled)
    }

    pub fn dependency_detail_hover_enabled(&self) -> bool {
        self.enabled()
            && self
                .hover()
                .map_or(true, PyprojectHoverFeatures::dependency_detail_enabled)
    }

    pub fn code_action_enabled(&self) -> bool {
        self.enabled()
            && self
                .code_action()
                .map_or(true, PyprojectCodeActionFeatures::enabled)
    }

    pub fn use_workspace_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                PyprojectCodeActionFeatures::use_workspace_dependency_enabled,
            )
    }

    pub fn add_to_workspace_and_use_workspace_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                PyprojectCodeActionFeatures::add_to_workspace_and_use_workspace_dependency_enabled,
            )
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
pub struct PyprojectLspFeatureTree {
    /// # Completion feature options
    ///
    /// Configure pyproject completion features.
    pub completion: Option<PyprojectCompletionFeatures>,

    /// # Inlay hint feature options
    ///
    /// Configure pyproject inlay hint features.
    pub inlay_hint: Option<PyprojectInlayHintFeatures>,

    /// # Goto definition feature options
    ///
    /// Configure pyproject go-to-definition features.
    pub goto_definition: Option<PyprojectNavigationFeatures>,

    /// # Goto declaration feature options
    ///
    /// Configure pyproject go-to-declaration features.
    pub goto_declaration: Option<PyprojectNavigationFeatures>,

    /// # Document link feature options
    ///
    /// Configure pyproject document link features.
    pub document_link: Option<PyprojectDocumentLinkFeatures>,

    /// # Hover feature options
    ///
    /// Configure pyproject hover features.
    pub hover: Option<PyprojectHoverFeatures>,

    /// # Code action feature options
    ///
    /// Configure pyproject code action features.
    pub code_action: Option<PyprojectCodeActionFeatures>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub enum PyprojectInlayHintFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectInlayHintFeatureTree),
}

default_to_features!(PyprojectInlayHintFeatures, PyprojectInlayHintFeatureTree);

impl PyprojectInlayHintFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn dependency_version_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency_version
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
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
pub struct PyprojectInlayHintFeatureTree {
    /// # Dependency version inlay hint feature
    ///
    /// Whether inlay hints show resolved dependency versions.
    pub dependency_version: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "jsonschema",
    schemars(extend(
        "x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending
    ))
)]
pub enum PyprojectHoverFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectHoverFeatureTree),
}

default_to_features!(PyprojectHoverFeatures, PyprojectHoverFeatureTree);

impl PyprojectHoverFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn dependency_detail_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency_detail
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
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
pub struct PyprojectHoverFeatureTree {
    /// # Dependency detail hover feature
    ///
    /// Whether hover shows detailed dependency metadata.
    pub dependency_detail: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectCompletionFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectCompletionFeatureTree),
}

default_to_features!(PyprojectCompletionFeatures, PyprojectCompletionFeatureTree);

impl PyprojectCompletionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn path_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => {
                    features.path.as_ref().map_or(true, ToggleFeature::enabled)
                }
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
pub struct PyprojectCompletionFeatureTree {
    /// # Path completion feature
    ///
    /// Whether completion suggests filesystem paths.
    pub path: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectNavigationFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectNavigationFeatureTree),
}

default_to_features!(PyprojectNavigationFeatures, PyprojectNavigationFeatureTree);

impl PyprojectNavigationFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn dependency_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .dependency
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn member_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .member
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn path_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => {
                    features.path.as_ref().map_or(true, ToggleFeature::enabled)
                }
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
pub struct PyprojectNavigationFeatureTree {
    /// # Dependency navigation feature
    ///
    /// Whether navigation resolves dependency definitions and declarations.
    pub dependency: Option<ToggleFeature>,

    /// # Member navigation feature
    ///
    /// Whether navigation resolves workspace member targets.
    pub member: Option<ToggleFeature>,

    /// # Path navigation feature
    ///
    /// Whether navigation resolves filesystem paths.
    pub path: Option<ToggleFeature>,
}

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
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn pyproject_toml_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .pyproject_toml
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn pypi_org_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .pypi_org
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
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
    pub pyproject_toml: Option<ToggleFeature>,

    /// # PyPI document link feature
    ///
    /// Whether document links are created for `pypi.org` package references.
    pub pypi_org: Option<ToggleFeature>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PyprojectCodeActionFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectCodeActionFeatureTree),
}

default_to_features!(PyprojectCodeActionFeatures, PyprojectCodeActionFeatureTree);

impl PyprojectCodeActionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn use_workspace_dependency_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .use_workspace_dependency
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
            }
    }

    pub fn add_to_workspace_and_use_workspace_dependency_enabled(&self) -> bool {
        self.enabled()
            && match self {
                Self::Enabled(_) => true,
                Self::Features(features) => features
                    .add_to_workspace_and_use_workspace_dependency
                    .as_ref()
                    .map_or(true, ToggleFeature::enabled),
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
pub struct PyprojectCodeActionFeatureTree {
    /// # Use-workspace-dependency code action feature
    ///
    /// Whether code actions can reuse a dependency declared in the workspace.
    pub use_workspace_dependency: Option<ToggleFeature>,

    /// # Add-to-workspace-and-use-workspace-dependency code action feature
    ///
    /// Whether code actions can add a dependency to the workspace and reuse it.
    pub add_to_workspace_and_use_workspace_dependency: Option<ToggleFeature>,
}
