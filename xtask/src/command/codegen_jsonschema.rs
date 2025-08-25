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
                    tombi_comment_directive::BooleanValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/integer-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::IntegerValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/float-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::FloatValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/string-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::StringValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/offset-date-time-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::OffsetDateTimeValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-time-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalDateTimeValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalDateValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-time-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalTimeValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/array-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::ArrayValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/table-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::TableValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/boolean-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::BooleanKeyValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/integer-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::IntegerKeyValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/float-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::FloatKeyValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/string-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::WithKeyTombiCommentDirectiveRules<
                        tombi_comment_directive::StringValueTombiCommentDirectiveRules,
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
                    tombi_comment_directive::OffsetDateTimeKeyValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-time-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalDateTimeKeyValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-date-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalDateKeyValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/local-time-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::LocalTimeKeyValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/array-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::ArrayKeyValueTombiCommentDirectiveRules,
                >>(),
        )? + "\n",
    )?;

    std::fs::write(
        project_root_path().join("json.tombi.dev/table-key-value-tombi-directive.json"),
        serde_json::to_string_pretty(
            &generator
                .clone()
                .into_root_schema_for::<tombi_comment_directive::ValueTombiCommentDirective<
                    tombi_comment_directive::TableKeyValueTombiCommentDirectiveRules,
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
