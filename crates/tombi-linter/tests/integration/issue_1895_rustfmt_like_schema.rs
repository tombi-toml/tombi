use tombi_linter::test_lint;
use tombi_test_lib::issue_1895_rustfmt_like_schema_path;

test_lint! {
    #[test]
    fn annotation_only_property_in_properties_is_allowed_in_strict_mode(
        r#"
        max_width = 120
        ignore = ["*_capnp.rs"]
        "#,
        SchemaPath(issue_1895_rustfmt_like_schema_path()),
    ) -> Ok(_)
}
