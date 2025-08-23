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

use tombi_severity_level::SeverityLevelDefaultError;

/// Common validation settings for all value types
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(deny_unknown_fields))]
pub struct CommonValueTombiCommentDirective {
    /// Controls the severity level for type mismatch errors
    pub type_mismatch: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for const value errors
    pub const_value: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for enumerate value errors
    pub enumerate: Option<SeverityLevelDefaultError>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/boolean-value-tombi-directive.json")))]
pub enum BooleanValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    Boolean(BooleanTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/integer-value-tombi-directive.json")))]
pub enum IntegerValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    Integer(IntegerTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/float-value-tombi-directive.json")))]
pub enum FloatValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    Float(FloatTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/string-value-tombi-directive.json")))]
pub enum StringValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    String(StringTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/offset-date-time-value-tombi-directive.json")))]
pub enum OffsetDateTimeValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    OffsetDateTime(OffsetDateTimeTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/local-date-time-value-tombi-directive.json")))]
pub enum LocalDateTimeValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    LocalDateTime(LocalDateTimeTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/local-date-value-tombi-directive.json")))]
pub enum LocalDateValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    LocalDate(LocalDateTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/local-time-value-tombi-directive.json")))]
pub enum LocalTimeValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    LocalTime(LocalTimeTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/array-value-tombi-directive.json")))]
pub enum ArrayValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    Array(ArrayTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/table-value-tombi-directive.json")))]
pub enum TableValueTombiCommentDirective {
    Common(CommonValueTombiCommentDirective),
    Table(TableTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/boolean-key-value-tombi-directive.json")))]
pub enum BooleanKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(BooleanTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/integer-key-value-tombi-directive.json")))]
pub enum IntegerKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(IntegerTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/float-key-value-tombi-directive.json")))]
pub enum FloatKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(FloatTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/string-key-value-tombi-directive.json")))]
pub enum StringKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(StringTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/offset-date-time-key-value-tombi-directive.json")))]
pub enum OffsetDateTimeKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(OffsetDateTimeTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/local-date-time-key-value-tombi-directive.json")))]
pub enum LocalDateTimeKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(LocalDateTimeTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/local-date-key-value-tombi-directive.json")))]
pub enum LocalDateKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(LocalDateTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/local-time-key-value-tombi-directive.json")))]
pub enum LocalTimeKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(LocalTimeTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/array-key-value-tombi-directive.json")))]
pub enum ArrayKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(ArrayTombiCommentDirective),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/table-key-value-tombi-directive.json")))]
pub enum TableKeyValueTombiCommentDirective {
    Key(KeyTombiCommentDirective),
    Value(TableTombiCommentDirective),
}
