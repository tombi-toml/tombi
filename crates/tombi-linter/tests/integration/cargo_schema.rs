use tombi_linter::test_lint;
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
