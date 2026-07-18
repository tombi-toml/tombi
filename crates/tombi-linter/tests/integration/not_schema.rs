use tombi_diagnostic::Level;
use tombi_linter::test_lint;

fn schema_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path()
        .join("schemas")
        .join("not-schema-test.schema.json")
}

test_lint! {
    #[test]
    fn test_invalid_ref_reports_warning_diagnostic(
        r#"
        value = "foo"
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_inline_not_rejects_matching_value(
        r#"
        inline = "foo"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::NotSchemaMatch
    ])
}

test_lint! {
    #[test]
    fn test_inline_not_accepts_non_matching_value(
        r#"
        inline = "bar"
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_inline_not_treats_warning_only_result_as_match(
        r#"
        warning = "foo"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::NotSchemaMatch
    ])
}
