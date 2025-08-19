mod boolean;
mod float;
mod integer;
mod local_date;
mod local_date_time;
mod local_time;
mod offset_date_time;
mod string;

pub use boolean::*;
pub use float::*;
pub use integer::*;
pub use local_date::*;
pub use local_date_time::*;
pub use local_time::*;
pub use offset_date_time::*;
pub use string::*;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "jsonschema", schemars(extend("$id" = "tombi://json.tombi.dev/value-tombi-directive.json")))]
pub enum ValueTombiCommentDirective {
    Boolean(BooleanTombiCommentDirective),
    Float(FloatTombiCommentDirective),
    Integer(IntegerTombiCommentDirective),
    LocalDateTime(LocalDateTimeTombiCommentDirective),
    LocalDate(LocalDateTombiCommentDirective),
    LocalTime(LocalTimeTombiCommentDirective),
    OffsetDateTime(OffsetDateTimeTombiCommentDirective),
    String(StringTombiCommentDirective),
}
