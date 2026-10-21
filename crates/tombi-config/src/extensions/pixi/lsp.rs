use crate::extensions::EnabledOnly;

mod completion;
mod hover;

pub use completion::*;
pub use hover::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub enum PixiLspFeatures {
    Enabled(EnabledOnly),
    Features(PixiLspFeatureTree),
}

extension_features! {
    PixiLspFeatures,

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
    pub struct PixiLspFeatureTree {
        /// # Completion feature options
        ///
        /// Configure pixi completion features.
        pub completion: Option<PixiCompletionFeatures>,

        /// # Hover feature options
        ///
        /// Configure pixi hover features.
        pub hover: Option<PixiHoverFeatures>,
    }
}
