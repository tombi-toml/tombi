use std::str::FromStr;

use tombi_severity_level::SeverityLevelDefaultError;
use tombi_uri::SchemaUri;

use crate::{TombiCommentDirectiveImpl, ValueTombiCommentDirective, WithCommonRules, WithKeyRules};

pub type FloatKeyValueTombiCommentDirective = ValueTombiCommentDirective<FloatKeyValueRules>;

pub type FloatValueTombiCommentDirective = ValueTombiCommentDirective<FloatValueRules>;

pub type FloatKeyValueRules = WithKeyRules<WithCommonRules<FloatRules>>;

pub type FloatValueRules = WithCommonRules<FloatRules>;

impl TombiCommentDirectiveImpl for FloatKeyValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/float-key-value-tombi-directive.json").unwrap()
    }
}

impl TombiCommentDirectiveImpl for FloatValueTombiCommentDirective {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://json.tombi.dev/float-value-tombi-directive.json").unwrap()
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct FloatRules {
    /// Controls the severity level for maximum float errors
    pub float_maximum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for minimum float errors
    pub float_minimum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive maximum float errors
    pub float_exclusive_maximum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for exclusive minimum float errors
    pub float_exclusive_minimum: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for multiple of float errors
    pub float_multiple_of: Option<SeverityLevelDefaultError>,
}
