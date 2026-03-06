use tombi_linter::test_lint;
use tombi_test_lib::dependent_schemas_test_schema_path;

test_lint! {
    #[test]
    fn test_dependent_schemas_satisfied(
        r#"
        use_ssl = true
        ssl_certificate = "cert.pem"
        ssl_key = "key.pem"
        "#,
        SchemaPath(dependent_schemas_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_dependent_schemas_not_satisfied(
        r#"
        use_ssl = true
        "#,
        SchemaPath(dependent_schemas_test_schema_path()),
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
    fn test_dependent_schemas_key_absent(
        r#"
        name = "John"
        ssl_certificate = "cert.pem"
        "#,
        SchemaPath(dependent_schemas_test_schema_path()),
    ) -> Ok(_)
}
