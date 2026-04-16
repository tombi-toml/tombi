use crate::extensions::EnabledOnly;

mod completion;
mod document_link;
mod goto_definition;
mod hover;

pub use completion::*;
pub use document_link::*;
pub use goto_definition::*;
pub use hover::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum TombiLspFeatures {
    Enabled(EnabledOnly),
    Features(TombiLspFeatureTree),
}

extension_features! {
    TombiLspFeatures,

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
        pub hover: Option<TombiHoverFeatures>,
    }
}
