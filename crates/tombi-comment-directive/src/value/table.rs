use std::str::FromStr;

use tombi_severity_level::{SeverityLevelDefaultError, SeverityLevelDefaultWarn};
use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type TableKeyValueTombiCommentDirective = ValueTombiCommentDirective<TableKeyValueRules>;

pub type TableValueTombiCommentDirective = ValueTombiCommentDirective<TableValueRules>;

pub type TableKeyValueRules = WithCommonRules<WithKeyRules<TableRules>>;

pub type TableValueRules = WithCommonRules<TableRules>;

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

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct TableRules {
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

    /// Controls the severity level for max properties errors
    pub table_max_properties: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for min properties errors
    pub table_min_properties: Option<SeverityLevelDefaultError>,
}
