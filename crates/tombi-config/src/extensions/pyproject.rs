use super::EnabledOnly;

mod lsp;

pub use lsp::*;

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
