use tombi_linter::test_lint;
use tombi_test_lib::unevaluated_properties_if_then_test_schema_path;

test_lint! {
    #[test]
    fn test_then_branch_annotations_apply_when_then_succeeds(
        r#"
        #:tombi schema.strict = false
        [then_branch]
        kind = "a"
        value = 1
        "#,
        SchemaPath(unevaluated_properties_if_then_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_failed_then_keeps_successful_if_annotations(
        r#"
        #:tombi schema.strict = false
        [then_branch]
        kind = "a"
        value = 1
        extra = true
        "#,
        SchemaPath(unevaluated_properties_if_then_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "value".to_string(),
        },
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "extra".to_string(),
        },
        tombi_validator::DiagnosticKind::TableMaxKeys {
            max_keys: 2,
            actual: 3,
        },
    ])
}

test_lint! {
    #[test]
    fn test_else_branch_annotations_apply_when_else_succeeds(
        r#"
        #:tombi schema.strict = false
        [else_branch]
        other = 1
        "#,
        SchemaPath(unevaluated_properties_if_then_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_else_branch_annotations_dropped_when_else_fails(
        r#"
        #:tombi schema.strict = false
        [else_branch]
        other = 1
        extra = 2
        "#,
        SchemaPath(unevaluated_properties_if_then_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "other".to_string(),
        },
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "extra".to_string(),
        },
        tombi_validator::DiagnosticKind::TableMaxKeys {
            max_keys: 1,
            actual: 2,
        },
    ])
}
