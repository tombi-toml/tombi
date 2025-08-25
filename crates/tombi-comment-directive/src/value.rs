mod array;
mod boolean;
mod float;
mod integer;
mod key;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod string;
mod table;

pub use array::*;
pub use boolean::*;
pub use float::*;
pub use integer::*;
pub use key::*;
pub use local_date::*;
pub use local_date_time::*;
pub use local_time::*;
pub use offset_date_time::*;
pub use string::*;
pub use table::*;

use tombi_schema_store::SchemaUri;
use tombi_severity_level::SeverityLevelDefaultError;

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(
    bound = "T: serde::de::DeserializeOwned + serde::Serialize + ValueTombiCommentDirectiveImpl"
)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct ValueTombiCommentDirective<T> {
    lint: Option<ValueLintOptions<T>>,
}

impl<T> From<ValueTombiCommentDirective<WithKeyTombiCommentDirectiveRules<T>>>
    for ValueTombiCommentDirective<T>
where
    T: From<WithKeyTombiCommentDirectiveRules<T>> + serde::de::DeserializeOwned + serde::Serialize,
{
    fn from(value: ValueTombiCommentDirective<WithKeyTombiCommentDirectiveRules<T>>) -> Self {
        Self {
            lint: value.lint.map(|lint| lint.into()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(bound = "T: serde::de::DeserializeOwned + serde::Serialize ")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct ValueLintOptions<T> {
    rules: Option<T>,
}

impl<T> From<ValueLintOptions<WithKeyTombiCommentDirectiveRules<T>>> for ValueLintOptions<T>
where
    T: serde::de::DeserializeOwned + serde::Serialize + From<WithKeyTombiCommentDirectiveRules<T>>,
{
    fn from(value: ValueLintOptions<WithKeyTombiCommentDirectiveRules<T>>) -> Self {
        Self {
            rules: value.rules.map(|rules| rules.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
#[cfg_attr(feature = "jsonschema", schemars(extend("x-tombi-table-keys-order" = tombi_x_keyword::TableKeysOrder::Ascending)))]
pub struct WithKeyTombiCommentDirectiveRules<T> {
    #[serde(flatten)]
    key: KeyTombiCommentDirectiveRules,

    #[serde(flatten)]
    value: T,
}

pub trait ValueTombiCommentDirectiveImpl {
    fn value_comment_directive_schema_url() -> SchemaUri;
}

#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct CommonValueTombiCommentDirectiveRules {
    /// Controls the severity level for type mismatch errors
    pub type_mismatch: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for const value errors
    pub const_value: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for enumerate value errors
    pub enumerate: Option<SeverityLevelDefaultError>,
}
