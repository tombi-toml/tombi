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

test_lint! {
    #[test]
    fn ref_applies_alongside_array_constraint(
        r#"
        ref_with_max_items = [1, 2, 3]
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayMaxValues {
            max_values: 2,
            actual: 3,
        },
    ])
}

test_lint! {
    #[test]
    fn ref_target_has_its_own_evaluation_scope(
        r#"
        [ref_creates_evaluation_scope]
        prop1 = "match"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "prop1".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn ref_annotations_contribute_to_sibling_unevaluated_properties(
        r#"
        [ref_with_unevaluated_properties]
        foo = "foo"
        bar = "bar"
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn ref_sibling_unevaluated_properties_rejects_unknown_property(
        r#"
        [ref_with_unevaluated_properties]
        foo = "foo"
        bar = "bar"
        baz = "baz"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "baz".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn external_ref_annotations_contribute_to_sibling_unevaluated_properties(
        r#"
        [external_ref_with_unevaluated_properties]
        foo = "foo"
        bar = "bar"
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn external_ref_sibling_unevaluated_properties_rejects_unknown_property(
        r#"
        [external_ref_with_unevaluated_properties]
        foo = "foo"
        bar = "bar"
        baz = "baz"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "baz".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn dynamic_ref_annotations_contribute_to_sibling_unevaluated_properties(
        r#"
        [dynamic_ref_with_unevaluated_properties]
        foo = "foo"
        bar = "bar"
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn dynamic_ref_sibling_unevaluated_properties_rejects_unknown_property(
        r#"
        [dynamic_ref_with_unevaluated_properties]
        foo = "foo"
        bar = "bar"
        baz = "baz"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::UnevaluatedPropertyNotAllowed {
            key: "baz".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn ref_annotations_contribute_to_sibling_unevaluated_items(
        r#"
        ref_with_unevaluated_items = [1]
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn ref_sibling_unevaluated_items_rejects_unknown_item(
        r#"
        ref_with_unevaluated_items = [1, 2]
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayUnevaluatedItemNotAllowed {
            index: 1,
        },
    ])
}

test_lint! {
    #[test]
    fn ref_target_has_its_own_array_evaluation_scope(
        r#"
        ref_creates_array_evaluation_scope = [1]
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayUnevaluatedItemNotAllowed {
            index: 0,
        },
    ])
}

test_lint! {
    #[test]
    fn failed_all_of_does_not_contribute_to_unevaluated_items(
        r#"
        failed_all_of_with_unevaluated_items = [1, 2]
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayUnevaluatedItemNotAllowed {
            index: 0,
        },
        tombi_validator::DiagnosticKind::ArrayUnevaluatedItemNotAllowed {
            index: 1,
        },
        tombi_validator::DiagnosticKind::ArrayMinValues {
            min_values: 5,
            actual: 2,
        },
    ])
}

test_lint! {
    #[test]
    fn failed_then_keeps_successful_if_annotations_for_unevaluated_items(
        r#"
        failed_conditional_with_unevaluated_items = [1, 2]
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayMinValues {
            min_values: 5,
            actual: 2,
        },
        tombi_validator::DiagnosticKind::ArrayUnevaluatedItemNotAllowed {
            index: 1,
        },
    ])
}
