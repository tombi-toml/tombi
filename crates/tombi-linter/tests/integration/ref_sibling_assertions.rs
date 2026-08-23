use tombi_linter::test_lint;
use tombi_test_lib::project_root_path;

fn schema_path() -> std::path::PathBuf {
    project_root_path()
        .join("crates/tombi-linter/tests/fixtures/ref-sibling-assertions/root.schema.json")
}

test_lint! {
    #[test]
    fn incompatible_type_const_and_enum_reject_values(
        r#"
        internal_type = 1
        internal_const = 1
        internal_enum = 1
        external_type = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::Nothing,
        tombi_validator::DiagnosticKind::Nothing,
        tombi_validator::DiagnosticKind::Nothing,
        tombi_validator::DiagnosticKind::Nothing,
    ])
}
