use tombi_linter::test_lint;
use tombi_test_lib::tuple_items_test_schema_path;

test_lint! {
    #[test]
    fn test_tuple_valid(
        r#"
        point = [1.0, 2.0]
        "#,
        SchemaPath(tuple_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_tuple_type_mismatch(
        r#"
        point = ["hello", 2.0]
        "#,
        SchemaPath(tuple_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::Float,
            actual: tombi_document_tree::ValueType::String,
        },
    ])
}

test_lint! {
    #[test]
    fn test_tuple_additional_items_false(
        r#"
        point = [1.0, 2.0, 3.0]
        "#,
        SchemaPath(tuple_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayAdditionalItems {
            max_items: 2,
        },
    ])
}

test_lint! {
    #[test]
    fn test_tuple_with_additional_items_schema_valid(
        r#"
        extensible = [1, "hello", "extra1", "extra2"]
        "#,
        SchemaPath(tuple_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_tuple_with_additional_items_schema_invalid(
        r#"
        extensible = [1, "hello", 42]
        "#,
        SchemaPath(tuple_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::String,
            actual: tombi_document_tree::ValueType::Integer,
        },
    ])
}

test_lint! {
    #[test]
    fn test_tuple_open_with_extra_items(
        r#"
        open_tuple = ["hello", 42, true, 3.14]
        "#,
        SchemaPath(tuple_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_tuple_record_valid(
        r#"
        record = ["Alice", 30, true]
        "#,
        SchemaPath(tuple_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_tuple_record_extra_item(
        r#"
        record = ["Alice", 30, true, "extra"]
        "#,
        SchemaPath(tuple_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayAdditionalItems {
            max_items: 3,
        },
    ])
}
