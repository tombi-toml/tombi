use schemars::{generate::SchemaSettings, SchemaGenerator};
use tombi_comment_directive::{
    ArrayKeyValueTombiCommentDirective, ArrayValueTombiCommentDirective,
    BooleanKeyValueTombiCommentDirective, BooleanValueTombiCommentDirective,
    FloatKeyValueTombiCommentDirective, FloatValueTombiCommentDirective,
    IntegerKeyValueTombiCommentDirective, IntegerValueTombiCommentDirective,
    KeyTombiCommentDirective, LocalDateKeyValueTombiCommentDirective,
    LocalDateTimeKeyValueTombiCommentDirective, LocalDateTimeValueTombiCommentDirective,
    LocalDateValueTombiCommentDirective, LocalTimeKeyValueTombiCommentDirective,
    LocalTimeValueTombiCommentDirective, OffsetDateTimeKeyValueTombiCommentDirective,
    OffsetDateTimeValueTombiCommentDirective, RootTableValueTombiCommentDirective,
    StringKeyValueTombiCommentDirective, StringValueTombiCommentDirective,
    TableKeyValueTombiCommentDirective, TableValueTombiCommentDirective,
    TombiDocumentCommentDirective,
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
                .into_root_schema_for::<TombiDocumentCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/boolean-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<BooleanValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/integer-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<IntegerValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/float-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<FloatValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/string-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<StringValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/offset-date-time-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<OffsetDateTimeValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-time-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<LocalDateTimeValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<LocalDateValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-time-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<LocalTimeValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/array-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<ArrayValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/table-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TableValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/boolean-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<BooleanKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/integer-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<IntegerKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/float-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<FloatKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/string-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<StringKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/offset-date-time-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<OffsetDateTimeKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-time-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<LocalDateTimeKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<LocalDateKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-time-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<LocalTimeKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/array-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<ArrayKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/table-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<TableKeyValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/root-table-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<RootTableValueTombiCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/key-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<KeyTombiCommentDirective>(),
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
    integer: Option<i64>,
    float: Option<f64>,
    string: Option<String>,
    array: Option<Vec<u64>>,
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
    Integer(i64),
    Float(f64),
    String(String),
    OffsetDateTime(chrono::DateTime<chrono::FixedOffset>),
    LocalDateTime(chrono::NaiveDateTime),
    LocalDate(chrono::NaiveDate),
    LocalTime(chrono::NaiveTime),
    Array(Vec<LiteralValue>),
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
struct ObjectValue {
    a: Option<i64>,
    b: Option<String>,
    c: Option<bool>,
}
