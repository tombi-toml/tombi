use crate::extensions::EnabledOnly;

mod code_action;
mod completion;
mod document_link;
mod hover;
mod inlay_hint;
mod navigation;

pub use code_action::*;
pub use completion::*;
pub use document_link::*;
pub use hover::*;
pub use inlay_hint::*;
pub use navigation::*;

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
