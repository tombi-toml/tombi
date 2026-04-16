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
pub enum CargoLspFeatures {
    Enabled(EnabledOnly),
    Features(CargoLspFeatureTree),
}

default_to_features!(CargoLspFeatures, CargoLspFeatureTree);

impl CargoLspFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn completion(&self) -> Option<&CargoCompletionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.completion.as_ref(),
        }
    }

    pub fn inlay_hint(&self) -> Option<&CargoInlayHintFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.inlay_hint.as_ref(),
        }
    }

    pub fn goto_definition(&self) -> Option<&CargoNavigationFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_definition.as_ref(),
        }
    }

    pub fn goto_declaration(&self) -> Option<&CargoNavigationFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_declaration.as_ref(),
        }
    }

    pub fn document_link(&self) -> Option<&CargoDocumentLinkFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.document_link.as_ref(),
        }
    }

    pub fn code_action(&self) -> Option<&CargoCodeActionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.code_action.as_ref(),
        }
    }

    pub fn hover(&self) -> Option<&CargoHoverFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.hover.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, CargoCompletionFeatures::enabled)
    }

    pub fn dependency_version_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, CargoCompletionFeatures::dependency_version_enabled)
    }

    pub fn dependency_feature_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, CargoCompletionFeatures::dependency_feature_enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, CargoCompletionFeatures::path_enabled)
    }

    pub fn inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, CargoInlayHintFeatures::enabled)
    }

    pub fn dependency_version_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, CargoInlayHintFeatures::dependency_version_enabled)
    }

    pub fn default_features_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, CargoInlayHintFeatures::default_features_enabled)
    }

    pub fn workspace_value_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .inlay_hint()
                .map_or(true, CargoInlayHintFeatures::workspace_value_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, CargoNavigationFeatures::enabled)
    }

    pub fn goto_definition_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, CargoNavigationFeatures::dependency_enabled)
    }

    pub fn goto_definition_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, CargoNavigationFeatures::member_enabled)
    }

    pub fn goto_definition_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, CargoNavigationFeatures::path_enabled)
    }

    pub fn goto_declaration_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, CargoNavigationFeatures::enabled)
    }

    pub fn goto_declaration_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, CargoNavigationFeatures::dependency_enabled)
    }

    pub fn goto_declaration_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, CargoNavigationFeatures::member_enabled)
    }

    pub fn goto_declaration_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_declaration()
                .map_or(true, CargoNavigationFeatures::path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::enabled)
    }

    pub fn cargo_toml_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::cargo_toml_enabled)
    }

    pub fn workspace_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::workspace_enabled)
    }

    pub fn git_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::git_enabled)
    }

    pub fn path_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::path_enabled)
    }

    pub fn hover_enabled(&self) -> bool {
        self.enabled() && self.hover().map_or(true, CargoHoverFeatures::enabled)
    }

    pub fn dependency_detail_hover_enabled(&self) -> bool {
        self.enabled()
            && self
                .hover()
                .map_or(true, CargoHoverFeatures::dependency_detail_enabled)
    }

    pub fn crates_io_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, CargoDocumentLinkFeatures::crates_io_enabled)
    }

    pub fn code_action_enabled(&self) -> bool {
        self.enabled()
            && self
                .code_action()
                .map_or(true, CargoCodeActionFeatures::enabled)
    }

    pub fn inherit_from_workspace_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                CargoCodeActionFeatures::inherit_from_workspace_enabled,
            )
    }

    pub fn inherit_dependency_from_workspace_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                CargoCodeActionFeatures::inherit_dependency_from_workspace_enabled,
            )
    }

    pub fn convert_dependency_to_table_format_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                CargoCodeActionFeatures::convert_dependency_to_table_format_enabled,
            )
    }

    pub fn add_to_workspace_and_inherit_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.code_action().map_or(
                true,
                CargoCodeActionFeatures::add_to_workspace_and_inherit_dependency_enabled,
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
pub struct CargoLspFeatureTree {
    /// # Completion feature options
    pub completion: Option<CargoCompletionFeatures>,

    /// # Inlay hint feature options
    pub inlay_hint: Option<CargoInlayHintFeatures>,

    /// # Goto definition feature options
    pub goto_definition: Option<CargoNavigationFeatures>,

    /// # Goto declaration feature options
    pub goto_declaration: Option<CargoNavigationFeatures>,

    /// # Document link feature options
    pub document_link: Option<CargoDocumentLinkFeatures>,

    /// # Hover feature options
    pub hover: Option<CargoHoverFeatures>,

    /// # Code action feature options
    pub code_action: Option<CargoCodeActionFeatures>,
}
