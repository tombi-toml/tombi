use std::{fs, path::Path};

use anyhow::{Context, Result};
use itertools::Either;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use tempfile::tempdir;
use tombi_config::TomlVersion;
use tombi_diagnostic::Level;
use tombi_linter::{LintOptions, Linter};
use tombi_schema_store::{
    AssociateSchemaOptions, Options as SchemaStoreOptions, SCHEMA_RESOLUTION_DIAGNOSTIC_CODE,
    SchemaStore, SchemaUri,
};

use crate::{
    convert::{json_object_to_toml_document, supports_instance},
    suite::{
        Draft, inject_dialect_if_missing, list_required_suite_files, remotes_dir,
        rewrite_localhost_remotes,
    },
};

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

#[derive(Debug, Default, Clone)]
pub struct SuiteSummary {
    pub files: usize,
    pub supported: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub failed_cases: Vec<FailedCase>,
}

#[derive(Debug, Clone)]
pub struct FailedCase {
    pub draft: String,
    pub file: String,
    pub case: String,
    pub test: String,
    pub expected_valid: bool,
    pub actual_valid: bool,
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub drafts: Vec<Draft>,
    pub filter: Option<String>,
    pub max_failures: Option<usize>,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            drafts: Draft::all().to_vec(),
            filter: None,
            max_failures: None,
        }
    }
}

pub async fn run_suite(suite_root: &Path, options: &RunOptions) -> Result<SuiteSummary> {
    let remotes = remotes_dir(suite_root);
    let mut summary = SuiteSummary::default();

    for draft in &options.drafts {
        let files = list_required_suite_files(suite_root, *draft)?;
        for file in files {
            if let Some(filter) = &options.filter {
                let display = file.to_string_lossy();
                if !display.contains(filter) {
                    continue;
                }
            }
            summary.files += 1;
            let file_summary = run_suite_file(*draft, &file, &remotes).await?;
            summary.supported += file_summary.supported;
            summary.passed += file_summary.passed;
            summary.failed += file_summary.failed;
            summary.skipped += file_summary.skipped;
            summary.failed_cases.extend(file_summary.failed_cases);

            if options
                .max_failures
                .is_some_and(|max| summary.failed >= max)
            {
                return Ok(summary);
            }
        }
    }

    Ok(summary)
}

async fn run_suite_file(draft: Draft, path: &Path, remotes: &Path) -> Result<SuiteSummary> {
    let cases: Vec<SuiteCase> = serde_json::from_slice(
        &fs::read(path).with_context(|| format!("failed to read {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse suite file {}", path.display()))?;

    let file_label = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown.json")
        .to_string();

    let mut summary = SuiteSummary::default();
    for case in cases {
        for test in case.tests {
            if !supports_instance(&test.data) {
                summary.skipped += 1;
                continue;
            }

            summary.supported += 1;
            let mut schema = case.schema.clone();
            inject_dialect_if_missing(&mut schema, draft);
            rewrite_localhost_remotes(&mut schema, remotes);

            let actual_valid = validate_case(&schema, &test.data).await.with_context(|| {
                format!(
                    "validation error in {} :: {} :: {}",
                    file_label, case.description, test.description
                )
            })?;

            if actual_valid == test.valid {
                summary.passed += 1;
            } else {
                summary.failed += 1;
                summary.failed_cases.push(FailedCase {
                    draft: draft.dir_name().to_string(),
                    file: file_label.clone(),
                    case: case.description.clone(),
                    test: test.description.clone(),
                    expected_valid: test.valid,
                    actual_valid,
                });
            }
        }
    }

    Ok(summary)
}

async fn validate_case(schema: &JsonValue, data: &JsonValue) -> Result<bool> {
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
        strict: Some(false.into()),
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
        Err(diagnostics) => {
            if diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code() == SCHEMA_RESOLUTION_DIAGNOSTIC_CODE)
            {
                // Schema never applied; do not treat as a successful validation.
                false
            } else {
                diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.level() != Level::ERROR)
            }
        }
    };

    Ok(actual_valid)
}

pub fn print_summary(summary: &SuiteSummary, verbose_failures: usize) {
    println!();
    println!("JSON Schema Test Suite (Definition A)");
    println!("  files:     {}", summary.files);
    println!("  supported: {}", summary.supported);
    println!("  passed:    {}", summary.passed);
    println!("  failed:    {}", summary.failed);
    println!("  skipped:   {} (non-TOML-representable)", summary.skipped);
    if summary.supported > 0 {
        let rate = (summary.passed as f64) * 100.0 / (summary.supported as f64);
        println!("  pass rate: {rate:.1}% of supported");
    }

    if !summary.failed_cases.is_empty() {
        println!();
        println!("Failures (showing up to {verbose_failures}):");
        for failure in summary.failed_cases.iter().take(verbose_failures) {
            println!(
                "  - {}/{} :: {} :: {} (expected valid={}, got valid={})",
                failure.draft,
                failure.file,
                failure.case,
                failure.test,
                failure.expected_valid,
                failure.actual_valid
            );
        }
        if summary.failed_cases.len() > verbose_failures {
            println!(
                "  ... and {} more",
                summary.failed_cases.len() - verbose_failures
            );
        }
    }
}
