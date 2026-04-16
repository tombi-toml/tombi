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
pub enum PyprojectLspFeatures {
    Enabled(EnabledOnly),
    Features(PyprojectLspFeatureTree),
}

extension_features! {
    PyprojectLspFeatures,

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
}
