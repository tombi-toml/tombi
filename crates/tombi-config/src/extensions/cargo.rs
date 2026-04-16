use crate::BoolDefaultTrue;

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
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }

    pub fn lsp(&self) -> Option<&CargoLspFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.lsp.as_ref(),
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
