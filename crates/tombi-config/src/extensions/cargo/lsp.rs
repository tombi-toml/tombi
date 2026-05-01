use crate::extensions::EnabledOnly;

mod code_action;
mod completion;
mod document_link;
mod goto_declaration;
mod goto_definition;
mod hover;
mod inlay_hint;
mod references;

pub use code_action::*;
pub use completion::*;
pub use document_link::*;
pub use goto_declaration::*;
pub use goto_definition::*;
pub use hover::*;
pub use inlay_hint::*;
pub use references::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum CargoLspFeatures {
    Enabled(EnabledOnly),
    Features(CargoLspFeatureTree),
}

extension_features! {
    CargoLspFeatures,

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
        /// # Code action feature options
        pub code_action: Option<CargoCodeActionFeatures>,

        /// # Completion feature options
        pub completion: Option<CargoCompletionFeatures>,

        /// # Document link feature options
        pub document_link: Option<CargoDocumentLinkFeatures>,

        /// # Goto declaration feature options
        pub goto_declaration: Option<CargoGotoDeclarationFeatures>,

        /// # Goto definition feature options
        pub goto_definition: Option<CargoGotoDefinitionFeatures>,

        /// # Hover feature options
        pub hover: Option<CargoHoverFeatures>,

        /// # Inlay hint feature options
        pub inlay_hint: Option<CargoInlayHintFeatures>,

        /// # References feature options
        pub references: Option<CargoReferencesFeatures>,
    }
}
