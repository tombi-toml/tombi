//! `$ref` with sibling object assertions (regression guards for issue #2151).
//!
//! Tombi projects a `$ref` that carries assertion siblings into an internal
//! `allOf` of the sibling-only schema and the reference target. The sibling-only
//! branch declares no `properties`, so strict mode must not close it on its own.
//! An explicit `additionalProperties: false` sibling, on the other hand, does
//! close the table for every key, because `additionalProperties` only sees the
//! `properties` of the schema object it appears in.

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
