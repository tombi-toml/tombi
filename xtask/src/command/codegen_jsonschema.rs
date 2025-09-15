use schemars::{generate::SchemaSettings, SchemaGenerator};
use tombi_comment_directive::document::TombiDocumentDirectiveContent;
use tombi_comment_directive::value::{
    TombiArrayDirectiveContent, TombiArrayOfTableDirectiveContent, TombiBooleanDirectiveContent,
    TombiFloatDirectiveContent, TombiInlineTableDirectiveContent, TombiIntegerDirectiveContent,
    TombiKeyArrayDirectiveContent, TombiKeyArrayOfTableDirectiveContent,
    TombiKeyBooleanDirectiveContent, TombiKeyDirectiveContent, TombiKeyFloatDirectiveContent,
    TombiKeyInlineTableDirectiveContent, TombiKeyIntegerDirectiveContent,
    TombiKeyLocalDateDirectiveContent, TombiKeyLocalDateTimeDirectiveContent,
    TombiKeyLocalTimeDirectiveContent, TombiKeyOffsetDateTimeDirectiveContent,
    TombiKeyStringDirectiveContent, TombiKeyTableDirectiveContent, TombiLocalDateDirectiveContent,
    TombiLocalDateTimeDirectiveContent, TombiLocalTimeDirectiveContent,
    TombiOffsetDateTimeDirectiveContent, TombiParentTableDirectiveContent,
    TombiRootTableDirectiveContent, TombiStringDirectiveContent, TombiTableDirectiveContent,
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
                .into_root_schema_for::<TombiBooleanDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-integer-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiIntegerDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-float-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiFloatDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-string-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiStringDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-offset-date-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiOffsetDateTimeDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-local-date-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiLocalDateTimeDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-local-date-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiLocalDateDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-local-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiLocalTimeDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-array-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiArrayDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-inline-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiInlineTableDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiTableDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-array-of-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiArrayOfTableDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-parent-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiParentTableDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-root-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiRootTableDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-boolean-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyBooleanDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-integer-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyIntegerDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-float-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyFloatDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-string-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyStringDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-offset-date-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyOffsetDateTimeDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-local-date-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyLocalDateTimeDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-local-date-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyLocalDateDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-local-time-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyLocalTimeDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-array-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyArrayDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-inline-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyInlineTableDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyTableDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-array-of-table-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyArrayOfTableDirectiveContent>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/tombi-key-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TombiKeyDirectiveContent>(),
        )? + "\n",
    )?;

    Ok(())
}

#[derive(Debug, Default, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
#[schemars(extend("x-tombi-toml-version" = TomlVersion::V1_1_0_Preview))]
#[schemars(extend("minProperties" = 1))]
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
    table: Option<TableValue>,
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

    Array(#[validate(length(min = 1, max = 10))] Vec<LiteralValue2>),
    Table(TableValue2),
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
#[schemars(extend("minProperties" = 2))]
struct TableValue {
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
    table: Option<TableValue2>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[schemars(extend("minProperties" = 2))]
struct TableValue2 {
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
