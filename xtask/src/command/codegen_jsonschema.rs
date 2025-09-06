use schemars::{generate::SchemaSettings, SchemaGenerator};
use tombi_comment_directive::document::TombiDocumentDirectiveContent;
use tombi_comment_directive::value::{
    ArrayCommonRules, ArrayKeyCommonRules, ArrayOfTableCommonRules, BooleanCommonRules,
    FloatCommonRules, InlineTableCommonRules, IntegerCommonRules, KeyArrayOfTableCommonRules,
    KeyBooleanCommonRules, KeyCommonExtensibleRules, KeyFloatCommonRules,
    KeyInlineTableCommonRules, KeyIntegerCommonRules, KeyLocalDateCommonRules,
    KeyLocalDateTimeCommonRules, KeyLocalTimeCommonRules, KeyOffsetDateTimeCommonRules,
    KeyStringCommonRules, KeyTableCommonRules, LocalDateCommonRules, LocalDateTimeCommonRules,
    LocalTimeCommonRules, OffsetDateTimeCommonRules, RootTableCommonRules, StringCommonRules,
    TableCommonRules, TombiValueDirectiveContent,
};
use tombi_config::TomlVersion;

use crate::utils::project_root_path;

pub fn run() -> Result<(), anyhow::Error> {
    let settings = SchemaSettings::draft07();
    let generator = SchemaGenerator::new(settings);

    std::fs::write(
        project_root_path().join("schemas/type-test.schema.json"),
        serde_json::to_string_pretty(&generator.clone().into_root_schema_for::<TypeTest>())? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.schemastore.org/tombi.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_config::Config>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-document-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiDocumentDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-boolean-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<BooleanCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-integer-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<IntegerCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-float-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<FloatCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-string-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<StringCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-offset-date-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<OffsetDateTimeCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-local-date-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<LocalDateTimeCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-local-date-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<LocalDateCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-local-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<LocalTimeCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-array-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<ArrayCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-inline-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<InlineTableCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<TableCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-array-of-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<ArrayOfTableCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-boolean-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyBooleanCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-integer-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyIntegerCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-float-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyFloatCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-string-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyStringCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-offset-date-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyOffsetDateTimeCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-local-date-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyLocalDateTimeCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-local-date-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyLocalDateCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-local-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyLocalTimeCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-array-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<ArrayKeyCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-inline-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyInlineTableCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyTableCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-array-of-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyArrayOfTableCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-root-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<RootTableCommonRules>>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiValueDirectiveContent<KeyCommonExtensibleRules>>(),
        )? + "\n",
    )?;

    Ok(())
}

#[derive(Debug, Default, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[schemars(extend("x-tombi-toml-version" = TomlVersion::V1_1_0_Preview))]
struct TypeTest {
    boolean: Option<bool>,
    #[validate(range(min = 1, max = 10))]
    integer: Option<i64>,
    #[validate(range(min = 1, max = 10))]
    float: Option<f64>,
    #[validate(length(min = 1, max = 10))]
    string: Option<String>,
    #[validate(length(min = 2, max = 10))]
    array: Option<Vec<LiteralValue>>,
    offset_date_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    local_date_time: Option<chrono::NaiveDateTime>,
    local_date: Option<chrono::NaiveDate>,
    local_time: Option<chrono::NaiveTime>,
    literal: Option<LiteralValue>,
    object: Option<ObjectValue>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
enum LiteralValue {
    Boolean(bool),
    Integer(#[validate(range(min = 1, max = 10))] i64),
    Float(#[validate(range(min = 1, max = 10))] f64),
    String(#[validate(length(min = 1, max = 10))] String),
    OffsetDateTime(chrono::DateTime<chrono::FixedOffset>),
    LocalDateTime(chrono::NaiveDateTime),
    LocalDate(chrono::NaiveDate),
    LocalTime(chrono::NaiveTime),
    Array(Vec<LiteralValue2>),
    Object(ObjectValue2),
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
enum LiteralValue2 {
    Boolean(bool),
    Integer(#[validate(range(min = 1, max = 10))] i64),
    Float(#[validate(range(min = 1, max = 10))] f64),
    String(#[validate(length(min = 1, max = 10))] String),
    OffsetDateTime(chrono::DateTime<chrono::FixedOffset>),
    LocalDateTime(chrono::NaiveDateTime),
    LocalDate(chrono::NaiveDate),
    LocalTime(chrono::NaiveTime),
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
struct ObjectValue {
    boolean: Option<bool>,
    #[validate(range(min = 1, max = 10))]
    integer: Option<i64>,
    #[validate(range(min = 1, max = 10))]
    float: Option<f64>,
    #[validate(length(min = 1, max = 10))]
    string: Option<String>,
    array: Option<Vec<LiteralValue2>>,
    offset_date_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    local_date_time: Option<chrono::NaiveDateTime>,
    local_date: Option<chrono::NaiveDate>,
    local_time: Option<chrono::NaiveTime>,
    literal: Option<LiteralValue2>,
    object: Option<ObjectValue2>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
struct ObjectValue2 {
    boolean: Option<bool>,
    #[validate(range(min = 1, max = 10))]
    integer: Option<i64>,
    #[validate(range(min = 1, max = 10))]
    float: Option<f64>,
    #[validate(length(min = 1, max = 10))]
    string: Option<String>,
    offset_date_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    local_date_time: Option<chrono::NaiveDateTime>,
    local_date: Option<chrono::NaiveDate>,
    local_time: Option<chrono::NaiveTime>,
}
