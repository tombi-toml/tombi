use tombi_linter::test_lint;
use tombi_severity_level::SeverityLevel;
use tombi_test_lib::cargo_schema_path;

test_lint! {
    #[test]
    fn test_workspace_dependencies(
        r#"
        [workspace.dependencies]
        serde.version = "^1.0.0"
        serde.features = ["derive"]
        serde.workspace = true
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_workspace_unknown(
        r#"
        [workspace]
        aaa = 1
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Err([tombi_validator::DiagnosticKind::TableStrictAdditionalKeys {
        accessors: tombi_schema_store::SchemaAccessors::from(vec![
            tombi_schema_store::SchemaAccessor::Key("workspace".to_string()),
        ]),
        schema_uri: tombi_schema_store::SchemaUri::from_file_path(cargo_schema_path()).unwrap(),
        key: "aaa".to_string(),
    }])
}

test_lint! {
    #[test]
    fn test_unknown_keys(
        r#"
        [aaa]
        bbb = 1
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Err([tombi_validator::DiagnosticKind::KeyNotAllowed { key: "aaa".to_string() }])
}

test_lint! {
    #[test]
    fn test_package_name_wrong_type(
        r#"
        [package]
        name = 1
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Err([tombi_validator::DiagnosticKind::TypeMismatch {
        expected: tombi_schema_store::ValueType::String,
        actual: tombi_document_tree::ValueType::Integer,
    }])
}

test_lint! {
    #[test]
    fn test_package_name_wrong_type_with_comment_directive_disabled_eq_true(
        r#"
        [package]
        name = 1 # tombi: lint.rules.type-mismatch.disabled = true
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_package_name_wrong_type_with_wrong_comment_directive_disabled_eq_true(
        r#"
        [package]
        name = 1 # tombi: lint.rules.type-mism.disabled = true
        "#,
        SchemaPath(cargo_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed { key: "type-mism".to_string() },
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::String,
            actual: tombi_document_tree::ValueType::Integer,
        }
    ])
}

async fn lint_deprecated_project_table(
    deprecated_level: Option<SeverityLevel>,
) -> Result<(), Vec<tombi_diagnostic::Diagnostic>> {
    let temp_dir = tempfile::tempdir().unwrap();
    let schema_path = temp_dir.path().join("schema.json");
    std::fs::write(
        &schema_path,
        r#"{
  "type": "object",
  "properties": {
    "value": {
      "type": "integer",
      "deprecated": true
    }
  },
  "additionalProperties": false
}
"#,
    )
    .unwrap();

    let schema_store = tombi_schema_store::SchemaStore::new();
    let schema_uri = tombi_schema_store::SchemaUri::from_file_path(&schema_path).unwrap();
    let mut config = tombi_config::Config::default();
    config.schemas = Some(vec![tombi_config::SchemaItem::Root(
        tombi_config::RootSchema {
            toml_version: None,
            path: schema_uri.to_string(),
            include: vec!["*.toml".to_string()],
            lint: deprecated_level.map(|enabled| tombi_config::SchemaLintOptions {
                rules: Some(tombi_config::SchemaLintRules {
                    deprecated: Some(tombi_config::SchemaDeprecatedRule {
                        enabled: Some(enabled),
                    }),
                }),
            }),
        },
    )]);
    schema_store.load_config(&config, None).await.unwrap();

    let source_path = temp_dir.path().join("test.toml");
    let lint_options = tombi_linter::LintOptions::default();
    let linter = tombi_linter::Linter::new(
        tombi_config::TomlVersion::default(),
        &lint_options,
        Some(itertools::Either::Right(source_path.as_path())),
        &schema_store,
    );

    linter.lint("value = 1\n").await
}

#[tokio::test]
async fn test_deprecated_schema_lint_level_off() {
    tombi_test_lib::init_log();

    let result = lint_deprecated_project_table(Some(SeverityLevel::Off)).await;
    pretty_assertions::assert_eq!(result, Ok(()));
}

#[tokio::test]
async fn test_deprecated_schema_lint_level_warn() {
    tombi_test_lib::init_log();

    let diagnostics = lint_deprecated_project_table(Some(SeverityLevel::Warn))
        .await
        .unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code(), "deprecated");
    assert!(diagnostics[0].is_warning());
}

#[tokio::test]
async fn test_deprecated_schema_lint_level_error() {
    tombi_test_lib::init_log();

    let diagnostics = lint_deprecated_project_table(Some(SeverityLevel::Error))
        .await
        .unwrap_err();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code(), "deprecated");
    assert!(diagnostics[0].is_error());
}
