use tombi_linter::test_lint;
use tombi_test_lib::project_root_path;

fn schema_path() -> std::path::PathBuf {
    project_root_path().join("schemas/issue-2191-validation-vocabulary.schema.json")
}

fn draft_2019_schema_path() -> std::path::PathBuf {
    project_root_path().join("schemas/issue-2191-validation-vocabulary-2019.schema.json")
}

fn custom_schema_path() -> std::path::PathBuf {
    project_root_path().join("schemas/issue-2191-custom-metaschema-usage.schema.json")
}

test_lint! {
    #[test]
    fn test_schema_vocabulary_does_not_disable_minimum(
        r#"
        value = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Err([tombi_validator::DiagnosticKind::FloatMinimum {
        minimum: 5.0,
        actual: 1.0,
    }])
}

test_lint! {
    #[test]
    fn test_embedded_custom_metaschema_overrides_parent_vocabulary(
        r#"
        [overridden]
        value = 1
        "#,
        SchemaPath(custom_schema_path()),
    ) -> Err([tombi_validator::DiagnosticKind::FloatMinimum {
        minimum: 5.0,
        actual: 1.0,
    }])
}

test_lint! {
    #[test]
    fn test_custom_metaschema_validation_vocabulary_disabled(
        r#"
        value = 1
        "#,
        SchemaPath(custom_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_custom_metaschema_vocabulary_is_inherited_by_embedded_resources(
        r#"
        [nested]
        value = 1
        "#,
        SchemaPath(custom_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_custom_metaschema_keeps_applicators(
        r#"
        known = true
        unknown = true
        "#,
        SchemaPath(custom_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed {
            key: "unknown".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_draft_2019_schema_vocabulary_does_not_disable_minimum(
        r#"
        value = 1
        "#,
        SchemaPath(draft_2019_schema_path()),
    ) -> Err([tombi_validator::DiagnosticKind::FloatMinimum {
        minimum: 5.0,
        actual: 1.0,
    }])
}

test_lint! {
    #[test]
    fn test_validation_vocabulary_disabled_keeps_applicators(
        r#"
        known = true
        unknown = true
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed {
            key: "unknown".to_string(),
        },
    ])
}
