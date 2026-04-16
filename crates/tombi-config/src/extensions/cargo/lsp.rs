use crate::{BoolDefaultTrue, extensions::EnabledOnly};

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
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }

    pub fn completion(&self) -> Option<CargoCompletionFeatures> {
        match self {
            Self::Enabled(enabled) => {
                if enabled.enabled.value() {
                    Some(CargoCompletionFeatures::default())
                } else {
                    Some(CargoCompletionFeatures::Enabled(enabled.clone()))
                }
            }
            Self::Features(features) => features.completion.clone(),
        }
    }

    pub fn inlay_hint(&self) -> Option<CargoInlayHintFeatures> {
        match self {
            Self::Enabled(enabled) => {
                if enabled.enabled.value() {
                    Some(CargoInlayHintFeatures::default())
                } else {
                    Some(CargoInlayHintFeatures::Enabled(enabled.clone()))
                }
            }
            Self::Features(features) => features.inlay_hint.clone(),
        }
    }

    pub fn goto_definition(&self) -> Option<CargoNavigationFeatures> {
        match self {
            Self::Enabled(enabled) => {
                if enabled.enabled.value() {
                    Some(CargoNavigationFeatures::default())
                } else {
                    Some(CargoNavigationFeatures::Enabled(enabled.clone()))
                }
            }
            Self::Features(features) => features.goto_definition.clone(),
        }
    }

    pub fn goto_declaration(&self) -> Option<CargoNavigationFeatures> {
        match self {
            Self::Enabled(enabled) => {
                if enabled.enabled.value() {
                    Some(CargoNavigationFeatures::default())
                } else {
                    Some(CargoNavigationFeatures::Enabled(enabled.clone()))
                }
            }
            Self::Features(features) => features.goto_declaration.clone(),
        }
    }

    pub fn document_link(&self) -> Option<CargoDocumentLinkFeatures> {
        match self {
            Self::Enabled(enabled) => {
                if enabled.enabled.value() {
                    Some(CargoDocumentLinkFeatures::default())
                } else {
                    Some(CargoDocumentLinkFeatures::Enabled(enabled.clone()))
                }
            }
            Self::Features(features) => features.document_link.clone(),
        }
    }

    pub fn code_action(&self) -> Option<CargoCodeActionFeatures> {
        match self {
            Self::Enabled(enabled) => {
                if enabled.enabled.value() {
                    Some(CargoCodeActionFeatures::default())
                } else {
                    Some(CargoCodeActionFeatures::Enabled(enabled.clone()))
                }
            }
            Self::Features(features) => features.code_action.clone(),
        }
    }

    pub fn hover(&self) -> Option<CargoHoverFeatures> {
        match self {
            Self::Enabled(enabled) => {
                if enabled.enabled.value() {
                    Some(CargoHoverFeatures::default())
                } else {
                    Some(CargoHoverFeatures::Enabled(enabled.clone()))
                }
            }
            Self::Features(features) => features.hover.clone(),
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
