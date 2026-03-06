use tombi_linter::test_lint;
use tombi_test_lib::untagged_union_schema_path;

test_lint! {
    #[test]
    fn test_untagged_union_schema(
        r#"
        #:schema schemas/untagged-union.schema.json

        favorite_color = "blue"
        "#,
        SchemaPath(untagged_union_schema_path()),
    ) -> Ok(_)
}
