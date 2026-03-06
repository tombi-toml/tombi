use tombi_linter::test_lint;
use tombi_test_lib::prefix_items_test_schema_path;

test_lint! {
    #[test]
    fn test_prefix_items_valid(
        r#"
        point = [1.0, 2.0]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_prefix_items_type_mismatch(
        r#"
        point = ["hello", 2.0]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::Float,
            actual: tombi_document_tree::ValueType::String,
        },
    ])
}

test_lint! {
    #[test]
    fn test_prefix_items_overflow_rejected(
        r#"
        point = [1.0, 2.0, 3.0]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayAdditionalItems {
            max_items: 2,
        },
    ])
}

test_lint! {
    #[test]
    fn test_prefix_items_with_overflow_schema_valid(
        r#"
        extensible = [1, "hello", "extra1", "extra2"]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_prefix_items_with_overflow_schema_invalid(
        r#"
        extensible = [1, "hello", 42]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::String,
            actual: tombi_document_tree::ValueType::Integer,
        },
    ])
}

test_lint! {
    #[test]
    fn test_prefix_items_open_with_extra(
        r#"
        open_tuple = ["hello", 42, true, 3.14]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Ok(_)
}
