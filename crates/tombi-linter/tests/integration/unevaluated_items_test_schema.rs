use tombi_linter::test_lint;
use tombi_test_lib::unevaluated_items_test_schema_path;

test_lint! {
    #[test]
    fn test_unevaluated_items_false_allows_only_prefix(
        r#"
        closed_tuple = [1]
        "#,
        SchemaPath(unevaluated_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_contains_marks_item_as_evaluated_for_unevaluated_items(
        r#"
        contains_only_closed = [1]
        "#,
        SchemaPath(unevaluated_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_unevaluated_items_false_rejects_overflow(
        r#"
        closed_tuple = [1, 2]
        "#,
        SchemaPath(unevaluated_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayUnevaluatedItemNotAllowed {
            index: 1,
        },
    ])
}

test_lint! {
    #[test]
    fn test_unevaluated_items_schema_accepts_matching_values(
        r#"
        typed_tuple = [1, "ok", "extra"]
        "#,
        SchemaPath(unevaluated_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_unevaluated_items_schema_rejects_type_mismatch(
        r#"
        typed_tuple = [1, 2]
        "#,
        SchemaPath(unevaluated_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::String,
            actual: tombi_document_tree::ValueType::Integer,
        },
    ])
}
