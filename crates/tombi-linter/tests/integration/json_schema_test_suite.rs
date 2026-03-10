use std::{
    fs,
    path::{Path, PathBuf},
};

use itertools::Either;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use tempfile::tempdir;
use tombi_config::TomlVersion;
use tombi_diagnostic::Level;
use tombi_linter::{LintOptions, Linter};
use tombi_schema_store::{
    AssociateSchemaOptions, Options as SchemaStoreOptions, SchemaStore, SchemaUri,
};
use tombi_test_lib::project_root_path;

#[derive(Debug, Deserialize)]
struct SuiteCase {
    description: String,
    schema: JsonValue,
    tests: Vec<SuiteTest>,
}

#[derive(Debug, Deserialize)]
struct SuiteTest {
    description: String,
    data: JsonValue,
    valid: bool,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SuiteSummary {
    supported: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    failed_cases: Vec<String>,
}

fn fixture_dir() -> PathBuf {
    project_root_path()
        .join("crates")
        .join("tombi-linter")
        .join("tests")
        .join("fixtures")
        .join("json-schema-test-suite")
}

fn is_toml_representable(value: &JsonValue) -> bool {
    match value {
        JsonValue::Null => false,
        JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => true,
        JsonValue::Array(items) => items.iter().all(is_toml_representable),
        JsonValue::Object(map) => map.iter().all(|(_, value)| is_toml_representable(value)),
    }
}

fn supports_case(test: &SuiteTest) -> bool {
    matches!(test.data, JsonValue::Object(_)) && is_toml_representable(&test.data)
}

async fn validate_case(
    schema: &JsonValue,
    data: &JsonValue,
) -> Result<bool, Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let schema_path = temp.path().join("schema.json");
    let source_path = temp.path().join("test.toml");

    fs::write(&schema_path, serde_json::to_vec_pretty(schema)?)?;
    let toml_text = json_object_to_toml_document(
        data.as_object()
            .expect("suite support check must guarantee object root"),
    );
    fs::write(&source_path, &toml_text)?;

    let schema_store = SchemaStore::new_with_options(SchemaStoreOptions {
        strict: Some(false),
        offline: Some(true),
        cache: None,
    });
    let schema_uri = SchemaUri::from_file_path(&schema_path)
        .expect("failed to convert suite schema path to schema uri");
    schema_store
        .associate_schema(
            schema_uri,
            vec!["*.toml".to_string()],
            &AssociateSchemaOptions::default(),
        )
        .await;

    let lint_options = LintOptions::default();
    let linter = Linter::new(
        TomlVersion::default(),
        &lint_options,
        Some(Either::Right(source_path.as_path())),
        &schema_store,
    );

    let actual_valid = match linter.lint(&toml_text).await {
        Ok(()) => true,
        Err(diagnostics) => diagnostics
            .iter()
            .all(|diagnostic| diagnostic.level() != Level::ERROR),
    };

    Ok(actual_valid)
}

fn json_object_to_toml_document(object: &serde_json::Map<String, JsonValue>) -> String {
    let mut out = String::new();
    for (key, value) in object {
        out.push_str(&toml_key(key));
        out.push_str(" = ");
        out.push_str(&json_value_to_toml(value));
        out.push('\n');
    }
    out
}

fn json_value_to_toml(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => unreachable!("null values are filtered by support checks"),
        JsonValue::Bool(boolean) => boolean.to_string(),
        JsonValue::Number(number) => number.to_string(),
        JsonValue::String(string) => serde_json::to_string(string).expect("string must serialize"),
        JsonValue::Array(items) => {
            let values = items.iter().map(json_value_to_toml).collect::<Vec<_>>();
            format!("[{}]", values.join(", "))
        }
        JsonValue::Object(object) => {
            let entries = object
                .iter()
                .map(|(key, value)| format!("{} = {}", toml_key(key), json_value_to_toml(value)))
                .collect::<Vec<_>>();
            format!("{{ {} }}", entries.join(", "))
        }
    }
}

fn toml_key(key: &str) -> String {
    if key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        && !key.is_empty()
    {
        key.to_string()
    } else {
        serde_json::to_string(key).expect("key must serialize")
    }
}

async fn run_suite_file(path: &Path) -> Result<SuiteSummary, Box<dyn std::error::Error>> {
    let cases: Vec<SuiteCase> = serde_json::from_slice(&fs::read(path)?)?;
    let mut summary = SuiteSummary::default();

    for case in cases {
        for test in case.tests {
            if !supports_case(&test) {
                summary.skipped += 1;
                continue;
            }

            summary.supported += 1;
            let actual_valid = validate_case(&case.schema, &test.data).await?;
            if actual_valid == test.valid {
                summary.passed += 1;
            } else {
                summary.failed += 1;
                summary
                    .failed_cases
                    .push(format!("{} :: {}", case.description, test.description));
            }
        }
    }

    Ok(summary)
}

async fn run_suite_files(paths: &[&str]) -> Result<SuiteSummary, Box<dyn std::error::Error>> {
    let mut total = SuiteSummary::default();
    for relative in paths {
        let summary = run_suite_file(&fixture_dir().join(relative)).await?;
        total.supported += summary.supported;
        total.passed += summary.passed;
        total.failed += summary.failed;
        total.skipped += summary.skipped;
        total.failed_cases.extend(summary.failed_cases);
    }
    Ok(total)
}

#[tokio::test]
async fn draft7_official_subset_pass_rate() -> Result<(), Box<dyn std::error::Error>> {
    tombi_test_lib::init_log();

    let summary =
        run_suite_files(&["draft7/dependencies.json", "draft7/propertyNames.json"]).await?;

    assert_eq!(
        summary,
        SuiteSummary {
            supported: 49,
            passed: 49,
            failed: 0,
            skipped: 7,
            failed_cases: vec![],
        }
    );

    Ok(())
}

#[tokio::test]
async fn draft2019_09_official_subset_pass_rate() -> Result<(), Box<dyn std::error::Error>> {
    tombi_test_lib::init_log();

    let summary = run_suite_files(&["draft2019-09/unevaluatedProperties-selected.json"]).await?;

    assert_eq!(
        summary,
        SuiteSummary {
            supported: 8,
            passed: 8,
            failed: 0,
            skipped: 0,
            failed_cases: vec![],
        }
    );

    Ok(())
}

#[tokio::test]
async fn draft2020_12_official_subset_pass_rate() -> Result<(), Box<dyn std::error::Error>> {
    tombi_test_lib::init_log();

    let summary = run_suite_files(&[
        "draft2020-12/dependentRequired.json",
        "draft2020-12/dependentSchemas.json",
    ])
    .await?;

    assert_eq!(
        summary,
        SuiteSummary {
            supported: 33,
            passed: 33,
            failed: 0,
            skipped: 7,
            failed_cases: vec![],
        }
    );

    Ok(())
}
