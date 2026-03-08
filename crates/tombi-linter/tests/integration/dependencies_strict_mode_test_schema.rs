use tombi_linter::test_lint;
use tombi_test_lib::dependencies_strict_mode_test_schema_path;

test_lint! {
    #[test]
    fn test_schema_dependency_with_additional_properties_true_in_strict_mode(
        r#"
        trigger_with_additional_properties_true = true
        alpha = "ok"
        beta = "still-allowed"
        gamma = "still-allowed"
        "#,
        SchemaPath(dependencies_strict_mode_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_schema_dependency_without_additional_properties_in_strict_mode(
        r#"
        trigger_without_additional_properties = true
        beta = "ok"
        alpha = "still-allowed"
        gamma = "still-allowed"
        "#,
        SchemaPath(dependencies_strict_mode_test_schema_path()),
    ) -> Ok(_)
}
