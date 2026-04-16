use super::EnabledOnly;

mod lsp;

pub use lsp::*;

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
pub enum TombiExtensionFeatures {
    Enabled(EnabledOnly),
    Features(TombiExtensionFeatureTree),
}

default_to_features!(TombiExtensionFeatures, TombiExtensionFeatureTree);

impl TombiExtensionFeatures {
    pub fn enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => enabled.enabled(),
            Self::Features(_) => true,
        }
    }

    pub fn lsp(&self) -> Option<&TombiLspFeatures> {
        match self {
            Self::Enabled(_) => None,
            Self::Features(features) => features.lsp.as_ref(),
        }
    }

    pub fn completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::completion_enabled)
    }

    pub fn path_completion_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::path_completion_enabled)
    }

    pub fn goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::goto_definition_enabled)
    }

    pub fn path_goto_definition_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::path_goto_definition_enabled)
    }

    pub fn document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::document_link_enabled)
    }

    pub fn path_document_link_enabled(&self) -> bool {
        self.enabled()
            && self
                .lsp()
                .map_or(true, TombiLspFeatures::path_document_link_enabled)
    }

    pub fn hover_enabled(&self) -> bool {
        self.enabled() && self.lsp().map_or(true, TombiLspFeatures::hover_enabled)
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
pub struct TombiExtensionFeatureTree {
    /// # Tombi LSP feature options
    pub lsp: Option<TombiLspFeatures>,
}
