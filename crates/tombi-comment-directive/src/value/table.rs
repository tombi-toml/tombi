use std::str::FromStr;

use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};
use tombi_uri::SchemaUri;

use crate::value::{TombiValueDirective, WithCommonRules, WithKeyRules};
use crate::TombiCommentDirectiveImpl;

pub type TombiKeyTableDirective = TombiValueDirective<KeyTableCommonRules>;

pub type TombiTableDirective = TombiValueDirective<TableCommonRules>;

pub type TombiRootTableDirective = TombiValueDirective<RootTableCommonRules>;

pub type KeyTableCommonRules = WithKeyRules<WithCommonRules<TableRules>>;

pub type TableCommonRules = WithCommonRules<TableRules>;

pub type RootTableCommonRules = WithCommonRules<RootTableRules>;

impl TombiCommentDirectiveImpl for TombiKeyTableDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-key-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiTableDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/tombi-table-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TombiRootTableDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/root-tombi-table-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct TableRules {
    /// # Dotted keys out of order.
    ///
    /// Check if dotted keys are defined out of order.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// apple.type = "fruit"
    /// orange.type = "fruit"
    /// apple.skin = "thin"
    /// orange.skin = "thick"
    ///
    /// # RECOMMENDED
    /// apple.type = "fruit"
    /// apple.skin = "thin"
    /// orange.type = "fruit"
    /// orange.skin = "thick"
    /// ```
    pub dotted_keys_out_of_order: Option<SeverityLevelDefaultWarn>,

    /// # Maximum properties.
    ///
    /// Check if the table has more than the maximum number of properties.
    ///
    /// ```rust
    /// length(table) <= maximum
    /// ```
    ///
    pub table_max_properties: Option<SeverityLevelDefaultError>,

    /// # Minimum properties.
    ///
    /// Check if the table has less than the minimum number of properties.
    ///
    /// ```rust
    /// length(table) >= minimum
    /// ```
    ///
    pub table_min_properties: Option<SeverityLevelDefaultError>,
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct RootTableRules {
    /// # Tables out of order.
    ///
    /// Check if tables are defined out of order.
    ///
    /// ```toml
    /// # VALID BUT DISCOURAGED
    /// [fruit.apple]
    /// [animal]
    /// [fruit.orange]
    ///
    /// # RECOMMENDED
    /// [fruit.apple]
    /// [fruit.orange]
    /// [animal]
    /// ```
    pub tables_out_of_order: Option<SeverityLevelDefaultWarn>,

    #[serde(flatten)]
    pub table: TableRules,
}
