use tombi_linter::test_lint;
use tombi_test_lib::dependencies_test_schema_path;

test_lint! {
    #[test]
    fn test_property_dependency_satisfied(
        r#"
        credit_card = "1234-5678"
        billing_address = "123 Main St"
        shipping_address = "456 Oak Ave"
        "#,
        SchemaPath(dependencies_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_property_dependency_not_satisfied(
        r#"
        credit_card = "1234-5678"
        "#,
        SchemaPath(dependencies_test_schema_path()),
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
    fn test_property_dependency_key_absent(
        r#"
        name = "John"
        "#,
        SchemaPath(dependencies_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_schema_dependency_satisfied(
        r#"
        use_ssl = true
        ssl_certificate = "cert.pem"
        ssl_key = "key.pem"
        "#,
        SchemaPath(dependencies_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_schema_dependency_not_satisfied(
        r#"
        use_ssl = true
        "#,
        SchemaPath(dependencies_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TableKeyRequired {
            key: "ssl_certificate".to_string(),
        },
        tombi_validator::DiagnosticKind::TableKeyRequired {
            key: "ssl_key".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_schema_dependency_key_absent(
        r#"
        name = "John"
        ssl_certificate = "cert.pem"
        "#,
        SchemaPath(dependencies_test_schema_path()),
    ) -> Ok(_)
}
