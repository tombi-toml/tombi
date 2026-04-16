use crate::extensions::EnabledOnly;

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
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn completion(&self) -> Option<&TombiCompletionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.completion.as_ref(),
        }
    }

    pub fn goto_definition(&self) -> Option<&TombiGotoDefinitionFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.goto_definition.as_ref(),
        }
    }

    pub fn document_link(&self) -> Option<&TombiDocumentLinkFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.document_link.as_ref(),
        }
    }

    pub fn hover(&self) -> Option<&EnabledOnly> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.hover.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, TombiCompletionFeatures::enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .completion()
                .map_or(true, TombiCompletionFeatures::path_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, TombiGotoDefinitionFeatures::enabled)
    }

    pub fn path_goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .goto_definition()
                .map_or(true, TombiGotoDefinitionFeatures::path_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, TombiDocumentLinkFeatures::enabled)
    }

    pub fn path_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .document_link()
                .map_or(true, TombiDocumentLinkFeatures::path_enabled)
    }

    pub fn hover_enabled(&self) -> bool {
        self.enabled() && self.hover().map_or(true, EnabledOnly::enabled)
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
