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

test_lint! {
    #[test]
    fn ref_sibling_type_allows_properties_declared_by_target_in_strict_mode(
        r#"
        [settings]
        minimum_release_age_excludes = ["github:kjanat/actionlint"]
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn ref_sibling_type_still_rejects_undeclared_properties(
        r#"
        [settings]
        unknown = true
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "unknown".to_string(),
        },
    ])
}
