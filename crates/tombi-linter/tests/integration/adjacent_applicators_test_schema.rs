use std::path::PathBuf;

use tombi_linter::test_lint;
use tombi_test_lib::project_root_path;

fn schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("adjacent-applicators-test.schema.json")
}

test_lint! {
    #[test]
    fn test_inferred_const_string_any(
        r#"
        const_string_any = "foo"
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_inferred_const_string_any_rejects_invalid_value(
        r#"
        const_string_any = "bar"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::Const {
            expected: "\"foo\"".to_string(),
            actual: "\"bar\"".to_string(),
        },
        tombi_validator::DiagnosticKind::StringPattern {
            pattern: "^foo$".to_string(),
            actual: "\"bar\"".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_adjacent_all_of_with_boolean_base_constraints(
        r#"
        boolean_all = true
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_adjacent_all_of_rejects_invalid_boolean_branch(
        r#"
        boolean_all = false
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::Const {
            expected: "true".to_string(),
            actual: "false".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_adjacent_any_of_with_integer_base_constraints(
        r#"
        integer_any = 12
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_adjacent_any_of_rejects_when_integer_no_branch_matches(
        r#"
        integer_any = 11
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::Const {
            expected: "10".to_string(),
            actual: "11".to_string(),
        },
        tombi_validator::DiagnosticKind::Const {
            expected: "12".to_string(),
            actual: "11".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_adjacent_one_of_rejects_multiple_number_matches(
        r#"
        number_one = 1.5
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::OneOfMultipleMatch {
            valid_count: 2,
            total_count: 2,
        },
    ])
}

test_lint! {
    #[test]
    fn test_adjacent_any_of_with_string_base_constraints(
        r#"
        string_any = "foo"
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_adjacent_any_of_rejects_when_no_branch_matches(
        r#"
        string_any = "baz"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringPattern {
            pattern: "^foo".to_string(),
            actual: "\"baz\"".to_string(),
        },
        tombi_validator::DiagnosticKind::StringPattern {
            pattern: "^bar".to_string(),
            actual: "\"baz\"".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_adjacent_one_of_rejects_multiple_matches(
        r#"
        string_one = "foo"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::OneOfMultipleMatch {
            valid_count: 2,
            total_count: 2,
        },
    ])
}

test_lint! {
    #[test]
    fn test_adjacent_all_of_with_array_base_constraints(
        r#"
        array_all = ["x", "y"]
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_adjacent_all_of_rejects_missing_contains(
        r#"
        array_all = ["y", "z"]
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayContains,
    ])
}

test_lint! {
    #[test]
    fn test_adjacent_all_of_with_offset_date_time_base_constraints(
        r#"
        offset_date_time_all = 2024-01-15T10:30:00Z
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_adjacent_any_of_with_local_date_time_base_constraints(
        r#"
        local_date_time_any = 2024-01-16T10:30:00
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_adjacent_one_of_rejects_multiple_local_date_matches(
        r#"
        local_date_one = 2024-01-15
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::OneOfMultipleMatch {
            valid_count: 2,
            total_count: 2,
        },
    ])
}

test_lint! {
    #[test]
    fn test_adjacent_all_of_with_local_time_base_constraints(
        r#"
        local_time_all = 10:30:00
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_adjacent_object_all_of(
        r#"
        [object_all]
        foo = 1
        bar = "ok"
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_adjacent_object_any_of(
        r#"
        [object_any]
        kind = "x"
        alpha = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_adjacent_object_one_of(
        r#"
        [object_one]
        foo = 1
        bar = 2
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::OneOfMultipleMatch {
            valid_count: 2,
            total_count: 2,
        },
    ])
}

test_lint! {
    #[test]
    fn test_inferred_object_with_dependent_required_and_any_of(
        r#"
        [inferred_dependent_required_any]
        foo = 1
        bar = 2
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_inferred_object_with_dependent_required_and_any_of_rejects_missing_dependency(
        r#"
        [inferred_dependent_required_any]
        foo = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TableDependencyRequired {
            dependent_key: "foo".to_string(),
            required_key: "bar".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_inferred_object_with_dependent_schemas_and_all_of(
        r#"
        [inferred_dependent_schemas_all]
        foo = 1
        bar = 2
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_inferred_object_with_dependent_schemas_and_all_of_rejects_missing_required_key(
        r#"
        [inferred_dependent_schemas_all]
        foo = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TableKeyRequired {
            key: "bar".to_string(),
        },
    ])
}
