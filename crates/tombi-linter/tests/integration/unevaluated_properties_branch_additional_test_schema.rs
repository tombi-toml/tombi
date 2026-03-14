use tombi_linter::test_lint;
use tombi_test_lib::unevaluated_properties_branch_additional_test_schema_path;

test_lint! {
    #[test]
    fn test_additional_properties_in_branch_marks_key_as_evaluated(
        r#"
        foo = 1
        "#,
        SchemaPath(unevaluated_properties_branch_additional_test_schema_path()),
    ) -> Ok(_)
}
