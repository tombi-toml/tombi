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
        project_root_path().join("schemas/tombi.schema.json"),
        serde_json::to_string_pretty(&generator.into_root_schema_for::<tombi_config::Config>())?
            + "\n",
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
