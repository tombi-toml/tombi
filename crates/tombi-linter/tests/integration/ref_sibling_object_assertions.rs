//! `$ref` with sibling object assertions (regression guards for issue #2151).
//!
//! A sibling `type` assertion is already represented by the resolved schema
//! view, so it does not require rebuilding that view as a projected `allOf`.
//! Other assertion siblings still need their own annotation scope. In
//! particular, an explicit `additionalProperties: false` sibling closes the
//! table for every key because it only sees `properties` from the schema object
//! in which it appears.

use tombi_diagnostic::Level;
use tombi_linter::test_lint;
use tombi_test_lib::project_root_path;

fn schema_path() -> std::path::PathBuf {
    project_root_path()
        .join("crates/tombi-linter/tests/fixtures/ref-sibling-object-assertions/root.schema.json")
}

test_lint! {
    #[test]
    fn type_sibling_accepts_referenced_key(
        r#"
        [type_sibling]
        known = true
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn type_sibling_warns_unknown_key_in_strict_mode(
        r#"
        [type_sibling]
        unknown = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "table-strict-additional-keys", level: Level::WARNING },
    ])
}

test_lint! {
    #[test]
    fn type_sibling_keeps_warning_for_unknown_key_next_to_failing_key(
        r#"
        [type_sibling]
        known = "not boolean"
        unknown = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "type-mismatch", level: Level::ERROR },
        { code: "table-strict-additional-keys", level: Level::WARNING },
    ])
}

test_lint! {
    #[test]
    fn type_sibling_keeps_warning_in_nested_table_when_parent_key_fails(
        r#"
        [nested_sibling]
        known = "not boolean"

        [nested_sibling.child]
        unknown_child = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "type-mismatch", level: Level::ERROR },
        { code: "table-strict-additional-keys", level: Level::WARNING },
    ])
}

test_lint! {
    #[test]
    fn type_sibling_type_mismatch_does_not_warn_for_referenced_key(
        r#"
        [type_sibling]
        known = "not boolean"
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "type-mismatch", level: Level::ERROR },
    ])
}

test_lint! {
    #[test]
    fn type_sibling_required_failure_does_not_warn_for_referenced_key(
        r#"
        [type_sibling_required_target]
        known = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "table-key-required", level: Level::ERROR },
    ])
}

test_lint! {
    #[test]
    fn type_sibling_with_closed_target_accepts_referenced_key(
        r#"
        [type_sibling_closed_target]
        known = true
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn type_sibling_with_closed_target_rejects_unknown_key(
        r#"
        [type_sibling_closed_target]
        unknown = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "key-not-allowed", level: Level::ERROR },
    ])
}

test_lint! {
    #[test]
    fn type_sibling_with_unevaluated_target_accepts_referenced_key(
        r#"
        [type_sibling_unevaluated_target]
        known = true
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn type_sibling_with_unevaluated_target_rejects_unknown_key(
        r#"
        [type_sibling_unevaluated_target]
        unknown = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "unevaluated-property-not-allowed", level: Level::ERROR },
    ])
}

test_lint! {
    #[test]
    fn closed_sibling_rejects_every_key(
        r#"
        [closed_sibling]
        known = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "key-not-allowed", level: Level::ERROR },
    ])
}

test_lint! {
    #[test]
    fn closed_sibling_with_closed_target_rejects_every_key(
        r#"
        [closed_sibling_closed_target]
        known = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "key-not-allowed", level: Level::ERROR },
    ])
}

test_lint! {
    #[test]
    fn closed_sibling_without_type_rejects_every_key(
        r#"
        [closed_sibling_without_type]
        known = true
        "#,
        SchemaPath(schema_path()),
    ) -> Diagnostics([
        { code: "key-not-allowed", level: Level::ERROR },
    ])
}
