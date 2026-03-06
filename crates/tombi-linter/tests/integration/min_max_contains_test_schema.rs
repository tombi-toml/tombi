use tombi_linter::test_lint;
use tombi_test_lib::min_max_contains_test_schema_path;

test_lint! {
    #[test]
    fn test_min_contains_satisfied(
        r#"
        tags = ["important-a", "other", "important-b"]
        "#,
        SchemaPath(min_max_contains_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_min_contains_not_satisfied(
        r#"
        tags = ["important-a", "other"]
        "#,
        SchemaPath(min_max_contains_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayMinContains {
            min_contains: 2,
            actual: 1,
        },
    ])
}

test_lint! {
    #[test]
    fn test_max_contains_exceeded(
        r#"
        tags = ["important-a", "important-b", "important-c", "important-d", "important-e"]
        "#,
        SchemaPath(min_max_contains_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayMaxContains {
            max_contains: 4,
            actual: 5,
        },
    ])
}

test_lint! {
    #[test]
    fn test_min_max_contains_both_satisfied(
        r#"
        tags = ["important-a", "important-b", "important-c", "other"]
        "#,
        SchemaPath(min_max_contains_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_max_contains_only_satisfied(
        r#"
        limited = [2, 4, 1, 3]
        "#,
        SchemaPath(min_max_contains_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_max_contains_only_exceeded(
        r#"
        limited = [2, 4, 6, 1]
        "#,
        SchemaPath(min_max_contains_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayMaxContains {
            max_contains: 2,
            actual: 3,
        },
    ])
}

test_lint! {
    #[test]
    fn test_numbers_min_contains_satisfied(
        r#"
        numbers = [5, 15]
        "#,
        SchemaPath(min_max_contains_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_numbers_min_contains_not_satisfied(
        r#"
        numbers = [1, 5, 9]
        "#,
        SchemaPath(min_max_contains_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayMinContains {
            min_contains: 1,
            actual: 0,
        },
    ])
}
