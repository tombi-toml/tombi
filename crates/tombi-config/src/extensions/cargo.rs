use super::EnabledOnly;

mod lsp;

pub use lsp::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoExtensionFeatures {
    Enabled(EnabledOnly),
    Features(CargoExtensionFeatureTree),
}

default_to_features!(CargoExtensionFeatures, CargoExtensionFeatureTree);

impl CargoExtensionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn lsp(&self) -> Option<&CargoLspFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.lsp.as_ref(),
        }
    }

    pub fn lsp_enabled(&self) -> bool {
        self.enabled() && self.lsp().map_or(true, CargoLspFeatures::enabled)
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::completion_enabled)
    }

    pub fn dependency_version_completion_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::dependency_version_completion_enabled,
            )
    }

    pub fn dependency_feature_completion_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::dependency_feature_completion_enabled,
            )
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::path_completion_enabled)
    }

    pub fn inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::inlay_hint_enabled)
    }

    pub fn dependency_version_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::dependency_version_inlay_hint_enabled,
            )
    }

    pub fn default_features_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::default_features_inlay_hint_enabled)
    }

    pub fn workspace_value_inlay_hint_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::workspace_value_inlay_hint_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_definition_enabled)
    }

    pub fn goto_definition_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_definition_dependency_enabled)
    }

    pub fn goto_definition_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_definition_member_enabled)
    }

    pub fn goto_definition_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_definition_path_enabled)
    }

    pub fn goto_declaration_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_declaration_enabled)
    }

    pub fn goto_declaration_dependency_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_declaration_dependency_enabled)
    }

    pub fn goto_declaration_member_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_declaration_member_enabled)
    }

    pub fn goto_declaration_path_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::goto_declaration_path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::document_link_enabled)
    }

    pub fn cargo_toml_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::cargo_toml_document_link_enabled)
    }

    pub fn workspace_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::workspace_document_link_enabled)
    }

    pub fn git_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::git_document_link_enabled)
    }

    pub fn path_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::path_document_link_enabled)
    }

    pub fn hover_enabled(&self) -> bool {
        self.enabled() && self.lsp().map_or(true, CargoLspFeatures::hover_enabled)
    }

    pub fn dependency_detail_hover_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::dependency_detail_hover_enabled)
    }

    pub fn crates_io_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::crates_io_document_link_enabled)
    }

    pub fn code_action_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, CargoLspFeatures::code_action_enabled)
    }

    pub fn inherit_from_workspace_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::inherit_from_workspace_code_action_enabled,
            )
    }

    pub fn inherit_dependency_from_workspace_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::inherit_dependency_from_workspace_code_action_enabled,
            )
    }

    pub fn convert_dependency_to_table_format_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::convert_dependency_to_table_format_code_action_enabled,
            )
    }

    pub fn add_to_workspace_and_inherit_dependency_code_action_enabled(&self) -> bool {
        self.enabled()
            && self.lsp().map_or(
                true,
                CargoLspFeatures::add_to_workspace_and_inherit_dependency_code_action_enabled,
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
pub struct CargoExtensionFeatureTree {
    /// # Cargo LSP feature options
    pub lsp: Option<CargoLspFeatures>,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::BoolDefaultTrue;
    use crate::extensions::ToggleFeatureDefaultTrue;

    use super::{CargoDocumentLinkFeatureTree, CargoInlayHintFeatureTree};

    #[test]
    fn cargo_inlay_hint_feature_tree_deserializes_workspace_value_key() {
        let features: CargoInlayHintFeatureTree = serde_json::from_value(serde_json::json!({
            "workspace-value": {
                "enabled": false
            }
        }))
        .expect("workspace-value should deserialize");

        assert_eq!(
            features.workspace_value,
            Some(ToggleFeatureDefaultTrue {
                enabled: Some(BoolDefaultTrue::from(false)),
            })
        );
    }

    #[test]
    fn cargo_inlay_hint_feature_tree_serializes_workspace_value_key() {
        let value = serde_json::to_value(CargoInlayHintFeatureTree {
            dependency_version: None,
            default_features: None,
            workspace_value: Some(ToggleFeatureDefaultTrue {
                enabled: Some(BoolDefaultTrue::from(false)),
            }),
        })
        .expect("workspace-value should serialize");

        assert_eq!(
            value.get("workspace-value"),
            Some(&serde_json::json!({
                "enabled": false
            }))
        );
        assert!(value.get("workspace").is_none());
    }

    #[test]
    fn cargo_document_link_feature_tree_deserializes_workspace_key() {
        let features: CargoDocumentLinkFeatureTree = serde_json::from_value(serde_json::json!({
            "workspace": {
                "enabled": false
            }
        }))
        .expect("workspace should deserialize");

        assert_eq!(
            features.workspace,
            Some(ToggleFeatureDefaultTrue {
                enabled: Some(BoolDefaultTrue::from(false)),
            })
        );
    }

    #[test]
    fn cargo_document_link_feature_tree_serializes_workspace_key() {
        let value = serde_json::to_value(CargoDocumentLinkFeatureTree {
            cargo_toml: None,
            workspace: Some(ToggleFeatureDefaultTrue {
                enabled: Some(BoolDefaultTrue::from(false)),
            }),
            git: None,
            path: None,
            crates_io: None,
        })
        .expect("workspace should serialize");

        assert_eq!(
            value.get("workspace"),
            Some(&serde_json::json!({
                "enabled": false
            }))
        );
    }
}
