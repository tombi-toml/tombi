use tombi_linter::test_lint;
use tombi_test_lib::table_const_enum_test_schema_path;

test_lint! {
    #[test]
    fn test_table_const_valid(
        r#"
        [fixed_config]
        mode = "production"
        debug = false
        "#,
        SchemaPath(table_const_enum_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_table_const_invalid(
        r#"
        [fixed_config]
        mode = "development"
        debug = true
        "#,
        SchemaPath(table_const_enum_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::Const {
            expected: r#"{"mode": "production", "debug": false}"#.to_string(),
            actual: r#"{"mode": "development", "debug": true}"#.to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_table_enum_valid(
        r#"
        [env_config]
        env = "staging"
        port = 8080
        "#,
        SchemaPath(table_const_enum_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_table_enum_invalid(
        r#"
        [env_config]
        env = "testing"
        port = 9999
        "#,
        SchemaPath(table_const_enum_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::Enum {
            expected: vec![
                r#"{"env": "development", "port": 3000}"#.to_string(),
                r#"{"env": "staging", "port": 8080}"#.to_string(),
                r#"{"env": "production", "port": 443}"#.to_string(),
            ],
            actual: r#"{"env": "testing", "port": 9999}"#.to_string(),
        },
    ])
}
