use tombi_linter::test_lint;
use tombi_test_lib::additional_properties_branch_keys_test_schema_path;

test_lint! {
    #[test]
    fn test_branch_only_key_still_rejected_by_additional_properties_false(
        r#"
        foo = 1
        "#,
        SchemaPath(additional_properties_branch_keys_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed {
            key: "foo".to_string(),
        },
    ])
}
