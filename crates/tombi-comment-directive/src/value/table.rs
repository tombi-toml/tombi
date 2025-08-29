use std::str::FromStr;

use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};
use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type TableKeyValueTombiCommentDirective = ValueTombiCommentDirective<TableKeyValueRules>;

pub type TableValueTombiCommentDirective = ValueTombiCommentDirective<TableValueRules>;

pub type RootTableValueTombiCommentDirective = ValueTombiCommentDirective<RootTableValueRules>;

pub type TableKeyValueRules = WithKeyRules<WithCommonRules<TableRules>>;

pub type TableValueRules = WithCommonRules<TableRules>;

pub type RootTableValueRules = WithCommonRules<RootTableRules>;

impl TombiCommentDirectiveImpl for TableKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/table-key-value-tombi-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for TableValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/table-value-tombi-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for RootTableValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/root-table-value-tombi-directive.json").unwrap()
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

    /// Controls the severity level for max properties errors
    pub table_max_properties: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for min properties errors
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
