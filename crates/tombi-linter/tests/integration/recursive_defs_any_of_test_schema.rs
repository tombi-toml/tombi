use tombi_linter::test_lint;
use tombi_test_lib::recursive_defs_any_of_test_schema_path;

test_lint! {
    #[test]
    fn test_recursive_defs_any_of_mutual_ref_does_not_hang(
        r#"
        foo = 1
        "#,
        SchemaPath(recursive_defs_any_of_test_schema_path()),
    ) -> Ok(_)
}
