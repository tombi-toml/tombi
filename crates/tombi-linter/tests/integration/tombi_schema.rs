use tombi_linter::test_lint;
use tombi_test_lib::tombi_schema_path;

test_lint! {
    #[test]
    fn test_tombi_config_in_this_repository(
        include_str!("../../../../tombi.toml"),
        SchemaPath(tombi_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_tombi_schema_format_rules_array_bracket_space_width_eq_0(
        r#"
        [format.rules]
        array-bracket-space-width = 0
        "#,
        SchemaPath(tombi_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_tombi_schema_lint_rules_with_unknown_key(
        r#"
        [[schemas]]
        root = "tool.taskipy"
        path = "schemas/partial-taskipy.schema.json"
        include = ["pyproject.toml"]
        unknown = true
        "#,
        SchemaPath(tombi_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed {
            key: "unknown".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_tombi_schema_lint_rules_key_empty_undefined(
        r#"
        [lint.rules]
        key-empty = "undefined"
        "#,
        SchemaPath(tombi_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::Enum {
            expected: vec!["\"off\"".to_string(), "\"warn\"".to_string(), "\"error\"".to_string()],
            actual: "\"undefined\"".to_string()
        },
    ])
}

test_lint! {
    #[test]
    fn test_tombi_schema_extensions_lsp_feature_tree(
        r#"
        [extensions]
        "tombi-toml/tombi" = { lsp.document-link.path.enabled = false, lsp.hover.enabled = false }
        "tombi-toml/cargo" = { lsp.hover.dependency-detail.enabled = false }
        "tombi-toml/pyproject" = { lsp.hover.dependency-detail.enabled = false }
        "#,
        SchemaPath(tombi_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_tombi_schema_lsp_feature_tree(
        r#"
        [lsp]
        completion = { directive.enabled = false, schema.enabled = true, extension.enabled = false }
        hover = { directive.enabled = false, schema.enabled = true, extension.enabled = false }
        document-link = { schema.enabled = false, extension.enabled = true }
        goto-definition = { schema.enabled = false, extension.enabled = true }
        goto-declaration = { extension.enabled = false }
        goto-type-definition = { directive.enabled = false, schema.enabled = true }
        code-action = { dot-keys-to-inline-table.enabled = false, inline-table-to-dot-keys.enabled = true, extension.enabled = false }
        "#,
        SchemaPath(tombi_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_tombi_schema_lint_rules_deprecated(
        r#"
        [[schemas]]
        path = "tombi://www.schemastore.org/cargo.json"
        include = ["Cargo.toml"]

        [schemas.lint.rules]
        deprecated = "error"
        "#,
        SchemaPath(tombi_schema_path()),
    ) -> Ok(_)
}
