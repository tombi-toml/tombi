use tombi_diagnostic::Level;
use tombi_linter::test_lint;

fn schema_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path()
        .join("schemas")
        .join("schema-resolution-error-test.schema.json")
}

test_lint! {
    #[test]
    fn test_property_schema_resolution_error(
        r#"
        direct = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_all_of_schema_resolution_error(
        r#"
        all-of-error = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_any_of_schema_resolution_error(
        r#"
        any-of-error = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_one_of_schema_resolution_error(
        r#"
        one-of-error = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_if_schema_resolution_error(
        r#"
        if-error = {}
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_then_schema_resolution_error(
        r#"
        then-error = {}
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_else_schema_resolution_error(
        r#"
        else-error = {}
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_items_schema_resolution_error(
        r#"
        items-error = [1]
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_prefix_items_schema_resolution_error(
        r#"
        prefix-items-error = [1]
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_overflow_items_schema_resolution_error(
        r#"
        overflow-items-error = [1, 2]
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_overflow_items_schema_not_applied(
        r#"
        overflow-items-error = [1]
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_contains_schema_resolution_error(
        r#"
        contains-error = [1]
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_unevaluated_items_schema_resolution_error(
        r#"
        unevaluated-items-error = [1]
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_pattern_property_schema_resolution_error(
        r#"
        [pattern-properties-error]
        pat-key = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_additional_property_schema_resolution_error(
        r#"
        [additional-properties-error]
        extra = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_unevaluated_property_schema_resolution_error(
        r#"
        [unevaluated-properties-error]
        extra = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING },
        { code: "table-strict-additional-keys", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_property_names_schema_resolution_error(
        r#"
        [property-names-error]
        key = 1
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_dependency_schema_resolution_error(
        r#"
        [dependencies-error]
        trigger = "x"
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_dependent_schema_resolution_error(
        r#"
        [dependent-schemas-error]
        trigger = "x"
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "schema-resolution", level: Level::WARNING }
    ])
}

test_lint! {
    #[test]
    fn test_nested_schema_resolution_warning_disabled(
        r#"
        direct = 1 # tombi: lint.rules.schema-resolution.disabled = true
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_schema_resolution_warning_disabled_keeps_other_diagnostics(
        r#"
        [unevaluated-properties-error]
        extra = 1 # tombi: lint.rules.schema-resolution.disabled = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "table-strict-additional-keys", level: Level::WARNING }
    ])
}
