use crate::{BoolDefaultTrue, extensions::EnabledOnly};

mod completion;
mod document_link;
mod goto_definition;

pub use completion::*;
pub use document_link::*;
pub use goto_definition::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum TombiLspFeatures {
    Enabled(EnabledOnly),
    Features(TombiLspFeatureTree),
}

default_to_features!(TombiLspFeatures, TombiLspFeatureTree);

impl TombiLspFeatures {
    pub fn enabled(&self) -> BoolDefaultTrue {
        match self {
            Self::Enabled(enabled) => enabled.enabled,
            Self::Features(_) => Default::default(),
        }
    }

    pub fn completion(&self) -> Option<TombiCompletionFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                TombiCompletionFeatures::default()
            } else {
                TombiCompletionFeatures::Enabled(enabled.clone())
            }),
            Self::Features(features) => features.completion.clone(),
        }
    }

    pub fn goto_definition(&self) -> Option<TombiGotoDefinitionFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                TombiGotoDefinitionFeatures::default()
            } else {
                TombiGotoDefinitionFeatures::Enabled(enabled.clone())
            }),
            Self::Features(features) => features.goto_definition.clone(),
        }
    }

    pub fn document_link(&self) -> Option<TombiDocumentLinkFeatures> {
        match self {
            Self::Enabled(enabled) => Some(if enabled.enabled.value() {
                TombiDocumentLinkFeatures::default()
            } else {
                TombiDocumentLinkFeatures::Enabled(enabled.clone())
            }),
            Self::Features(features) => features.document_link.clone(),
        }
    }

    pub fn hover(&self) -> Option<EnabledOnly> {
        match self {
            Self::Enabled(enabled) => Some(enabled.clone()),
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
pub struct TombiLspFeatureTree {
    /// # Completion feature options
    ///
    /// Configure Tombi completion features.
    pub completion: Option<TombiCompletionFeatures>,

    /// # Goto definition feature options
    ///
    /// Configure Tombi go-to-definition features.
    pub goto_definition: Option<TombiGotoDefinitionFeatures>,

    /// # Document link feature options
    ///
    /// Configure Tombi document link features.
    pub document_link: Option<TombiDocumentLinkFeatures>,

    /// # Hover feature options
    ///
    /// Configure Tombi hover features.
    pub hover: Option<EnabledOnly>,
}
