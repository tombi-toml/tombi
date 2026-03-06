use tombi_linter::test_lint;
use tombi_test_lib::dependent_required_test_schema_path;

test_lint! {
    #[test]
    fn test_dependent_required_satisfied(
        r#"
        credit_card = "1234-5678"
        billing_address = "123 Main St"
        shipping_address = "456 Oak Ave"
        "#,
        SchemaPath(dependent_required_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_dependent_required_not_satisfied(
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
    fn test_dependent_required_key_absent(
        r#"
        name = "John"
        "#,
        SchemaPath(dependent_required_test_schema_path()),
    ) -> Ok(_)
}
