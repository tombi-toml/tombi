use super::EnabledOnly;

mod lint;
mod lsp;

pub use lint::*;
pub use lsp::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum NagiSqlExtensionFeatures {
    Enabled(EnabledOnly),
    Features(NagiSqlExtensionFeatureTree),
}

extension_features! {
    NagiSqlExtensionFeatures,

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
    pub struct NagiSqlExtensionFeatureTree {
        /// # Nagi SQL lint options
        #[cfg_attr(feature = "jsonschema", schemars(skip))]
        pub lint: Option<NagiSqlLintFeatures>,

        /// # Nagi SQL LSP feature options
        pub lsp: Option<NagiSqlLspFeatures>,
    }
}
