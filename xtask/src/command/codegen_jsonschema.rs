use schemars::{generate::SchemaSettings, SchemaGenerator};
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
        project_root_path().join("json.tombi.dev/document-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::TombiDocumentCommentDirective>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/boolean-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::BooleanValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/integer-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::IntegerValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/float-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::FloatValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/string-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::StringValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/offset-date-time-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::OffsetDateTimeValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-time-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalDateTimeValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalDateValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-time-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalTimeValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/array-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::ArrayValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/table-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::TableValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/boolean-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::BooleanKeyValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/integer-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::IntegerKeyValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/float-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::FloatKeyValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/string-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::WithKeyRules<
                        tombi_comment_directive::StringValueRules,
                    >,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/offset-date-time-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::OffsetDateTimeKeyValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-time-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalDateTimeKeyValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalDateKeyValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-time-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalTimeKeyValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/array-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::ArrayKeyValueRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/table-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::TableKeyValueRules,
                >>(),
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
