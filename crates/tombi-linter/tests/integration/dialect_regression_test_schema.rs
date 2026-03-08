use tombi_linter::test_lint;
use tombi_test_lib::{
    dependent_required_test_schema_path, prefix_items_test_schema_path, tuple_items_test_schema_path,
};

test_lint! {
    #[test]
    fn dialect_regression_draft07_tuple_items_rejects_overflow(
        r#"
        point = [1.0, 2.0, 3.0]
        "#,
        SchemaPath(tuple_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayAdditionalItems {
            max_items: 2,
        },
    ])
}

test_lint! {
    #[test]
    fn dialect_regression_2019_09_dependent_required_enforced(
        r#"
        credit_card = "1234-5678"
        "#,
        SchemaPath(dependent_required_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TableDependencyRequired {
            dependent_key: "credit_card".to_string(),
            required_key: "billing_address".to_string(),
        },
        tombi_validator::DiagnosticKind::TableDependencyRequired {
            dependent_key: "credit_card".to_string(),
            required_key: "shipping_address".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn dialect_regression_2020_12_prefix_items_rejects_overflow(
        r#"
        point = [1.0, 2.0, 3.0]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::ArrayAdditionalItems {
            max_items: 2,
        },
    ])
}
