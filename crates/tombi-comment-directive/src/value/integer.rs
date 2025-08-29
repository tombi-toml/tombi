use std::str::FromStr;

use tombi_severity_level::SeverityLevelDefaultError;
use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type IntegerKeyValueTombiCommentDirective = ValueTombiCommentDirective<IntegerKeyValueRules>;

pub type IntegerValueTombiCommentDirective = ValueTombiCommentDirective<IntegerValueRules>;

pub type IntegerKeyValueRules = WithKeyRules<WithCommonRules<IntegerRules>>;

pub type IntegerValueRules = WithCommonRules<IntegerRules>;

impl TombiCommentDirectiveImpl for IntegerKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/integer-key-value-tombi-directive.json")
            .unwrap()
    }
}

impl TombiCommentDirectiveImpl for IntegerValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/integer-value-tombi-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct IntegerRules {
    /// Controls the severity level for maximum integer errors
    pub integer_maximum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum integer errors
    pub integer_minimum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive maximum integer errors
    pub integer_exclusive_maximum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive minimum integer errors
    pub integer_exclusive_minimum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for multiple of integer errors
    pub integer_multiple_of: Option<SeverityLevelDefaultError>,
}
