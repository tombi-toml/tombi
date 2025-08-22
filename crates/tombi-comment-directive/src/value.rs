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
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
pub struct ValueCommonTombiCommentDirective {
    /// Controls the severity level for type mismatch errors
    pub type_mismatch: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for const value errors
    pub const_value: Option<SeverityLevelDefaultError>,

    /// Controls the severity level for enumerate value errors
    pub enumerate: Option<SeverityLevelDefaultError>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/value-tombi-directive.json")))]
pub enum ValueTombiCommentDirective {
    Array(ArrayTombiCommentDirective),
    Boolean(BooleanTombiCommentDirective),
    Float(FloatTombiCommentDirective),
    Integer(IntegerTombiCommentDirective),
    Key(KeyTombiCommentDirective),
    LocalDateTime(LocalDateTimeTombiCommentDirective),
    LocalDate(LocalDateTombiCommentDirective),
    LocalTime(LocalTimeTombiCommentDirective),
    OffsetDateTime(OffsetDateTimeTombiCommentDirective),
    String(StringTombiCommentDirective),
    Table(TableTombiCommentDirective),
}
