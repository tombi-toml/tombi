use tombi_linter::test_lint;
use tombi_test_lib::contains_test_schema_path;

test_lint! {
    #[test]
    fn test_contains_satisfied(
        r#"
        tags = ["hello", "required-tag", "world"]
        "#,
        SchemaPath(contains_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_contains_not_satisfied(
        r#"
        tags = ["hello", "world"]
        "#,
        SchemaPath(contains_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayContains,
    ])
}

test_lint! {
    #[test]
    fn test_contains_empty_array(
        r#"
        tags = []
        "#,
        SchemaPath(contains_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayContains,
    ])
}

test_lint! {
    #[test]
    fn test_contains_with_minimum_satisfied(
        r#"
        numbers = [1, 2, 15, 3]
        "#,
        SchemaPath(contains_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_contains_with_minimum_not_satisfied(
        r#"
        numbers = [1, 2, 3]
        "#,
        SchemaPath(contains_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayContains,
    ])
}

test_lint! {
    #[test]
    fn test_contains_type_only_satisfied(
        r#"
        mixed = [1, "hello", true]
        "#,
        SchemaPath(contains_test_schema_path()),
    ) -> Ok(_)
}
