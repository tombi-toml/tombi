use tombi_linter::test_lint;
use tombi_test_lib::recursive_anchor_ref_test_schema_path;

test_lint! {
    #[test]
    fn test_recursive_anchor_ref_valid(
        r#"
        id = 1

        [child]
        id = 2

        [child.child]
        id = 3
        "#,
        SchemaPath(recursive_anchor_ref_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_recursive_anchor_ref_type_mismatch(
        r#"
        id = 1

        [child]
        id = "invalid"
        "#,
        SchemaPath(recursive_anchor_ref_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::Integer,
            actual: tombi_document_tree::ValueType::String,
        },
    ])
}
