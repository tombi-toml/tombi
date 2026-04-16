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
pub enum PyprojectLspFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectLspFeatureTree),
}

default_to_features!(PyprojectLspFeatures, PyprojectLspFeatureTree);

impl PyprojectLspFeatures {
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }

    pub fn completion(&self) -> Option<PyprojectCompletionFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                PyprojectCompletionFeatures::default()
            } else {
                PyprojectCompletionFeatures::Enabled(enabled.clone())
            }),
            Self::Features(features) => features.completion.clone(),
        }
    }

    pub fn inlay_hint(&self) -> Option<PyprojectInlayHintFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                PyprojectInlayHintFeatures::default()
            } else {
                PyprojectInlayHintFeatures::Enabled(enabled.clone())
            }),
            Self::Features(features) => features.inlay_hint.clone(),
        }
    }

    pub fn goto_definition(&self) -> Option<PyprojectNavigationFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                PyprojectNavigationFeatures::default()
            } else {
                PyprojectNavigationFeatures::Enabled(enabled.clone())
            }),
            Self::Features(features) => features.goto_definition.clone(),
        }
    }

    pub fn goto_declaration(&self) -> Option<PyprojectNavigationFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                PyprojectNavigationFeatures::default()
            } else {
                PyprojectNavigationFeatures::Enabled(enabled.clone())
            }),
            Self::Features(features) => features.goto_declaration.clone(),
        }
    }

    pub fn document_link(&self) -> Option<PyprojectDocumentLinkFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                PyprojectDocumentLinkFeatures::default()
            } else {
                PyprojectDocumentLinkFeatures::Enabled(enabled.clone())
            }),
            Self::Features(features) => features.document_link.clone(),
        }
    }

    pub fn code_action(&self) -> Option<PyprojectCodeActionFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                PyprojectCodeActionFeatures::default()
            } else {
                PyprojectCodeActionFeatures::Enabled(enabled.clone())
            }),
            Self::Features(features) => features.code_action.clone(),
        }
    }

    pub fn hover(&self) -> Option<PyprojectHoverFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                PyprojectHoverFeatures::default()
            } else {
                PyprojectHoverFeatures::Enabled(enabled.clone())
            }),
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
