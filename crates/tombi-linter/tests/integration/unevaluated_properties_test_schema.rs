use tombi_linter::test_lint;
use tombi_test_lib::unevaluated_properties_test_schema_path;

test_lint! {
    #[test]
    fn test_unevaluated_properties_false_rejects_unknown_key_without_additional_properties(
        r#"
        known = "ok"
        extra = 1
        "#,
        SchemaPath(unevaluated_properties_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "extra".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_unevaluated_properties_false_allows_known_key(
        r#"
        known = "ok"
        "#,
        SchemaPath(unevaluated_properties_test_schema_path()),
    ) -> Ok(_)
}
