use std::str::FromStr;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};
use tombi_severity_level::SeverityLevelDefaultError;
use tombi_uri::SchemaUri;

pub type ArrayKeyValueTombiCommentDirective = ValueTombiCommentDirective<ArrayKeyValueRules>;

pub type ArrayValueTombiCommentDirective = ValueTombiCommentDirective<ArrayValueRules>;

pub type ArrayKeyValueRules = WithKeyRules<ArrayRules>;

pub type ArrayValueRules = WithCommonRules<ArrayRules>;

impl TombiCommentDirectiveImpl for ArrayKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/array-key-value-tombi-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for ArrayValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/array-value-tombi-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct ArrayRules {
    /// Controls the severity level for max values errors
    pub array_max_items: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for min values errors
    pub array_min_items: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for unique values errors
    pub array_unique_items: Option<SeverityLevelDefaultError>,
}
