use tombi_linter::test_lint;
use tombi_test_lib::{
    union_best_match_any_of_test_schema_path, union_best_match_one_of_test_schema_path,
};

test_lint! {
    #[test]
    fn any_of_prefers_matching_discriminator(
        r#"
        kind = "bar"
        bar1 = "bar1"
        bar3 = "bar3"
        "#,
        SchemaPath(union_best_match_any_of_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed {
            key: "bar3".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn one_of_prefers_branch_with_more_declared_values(
        r#"
        foo1 = "foo1"
        foo3 = "foo3"
        "#,
        SchemaPath(union_best_match_one_of_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed {
            key: "foo3".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn one_of_combines_diagnostics_for_an_exact_tie(
        r#"
        foo1 = "foo1"
        bar2 = "bar2"
        "#,
        SchemaPath(union_best_match_one_of_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed {
            key: "bar2".to_string(),
        },
        tombi_validator::DiagnosticKind::KeyNotAllowed {
            key: "foo1".to_string(),
        },
    ])
}
