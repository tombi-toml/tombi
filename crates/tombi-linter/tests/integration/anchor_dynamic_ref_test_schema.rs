use tombi_linter::test_lint;
use tombi_test_lib::anchor_dynamic_ref_test_schema_path;

test_lint! {
    #[test]
    fn test_anchor_ref_and_dynamic_ref_valid(
        r#"
        name = "Alice"
        priority = 10
        "#,
        SchemaPath(anchor_dynamic_ref_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_anchor_ref_type_mismatch(
        r#"
        name = 42
        priority = 10
        "#,
        SchemaPath(anchor_dynamic_ref_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::String,
            actual: tombi_document_tree::ValueType::Integer,
        },
    ])
}

test_lint! {
    #[test]
    fn test_dynamic_ref_type_mismatch(
        r#"
        name = "Alice"
        priority = "high"
        "#,
        SchemaPath(anchor_dynamic_ref_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::Integer,
            actual: tombi_document_tree::ValueType::String,
        },
    ])
}
