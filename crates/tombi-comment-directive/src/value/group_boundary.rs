use std::str::FromStr;

use crate::TombiCommentDirectiveImpl;
use crate::value::TombiValueDirectiveContent;
use tombi_uri::SchemaUri;

pub type TombiGroupBoundaryDirectiveContent =
    TombiValueDirectiveContent<GroupBoundaryFormatRules, GroupBoundaryLintRules>;

impl TombiCommentDirectiveImpl for TombiGroupBoundaryDirectiveContent {
    fn comment_directive_schema_url() -> SchemaUri {
        SchemaUri::from_str("tombi://www.schemastore.tombi/tombi-group-boundary-directive.json")
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct GroupBoundaryFormatRules {}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct GroupBoundaryLintRules {}
