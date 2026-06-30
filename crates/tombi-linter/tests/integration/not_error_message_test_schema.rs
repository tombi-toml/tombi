use std::path::PathBuf;

use tombi_linter::test_lint;
use tombi_test_lib::project_root_path;

fn schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("not-error-message-test.schema.json")
}

test_lint! {
    #[test]
    fn test_not_schema_match_with_custom_error_message(
        r#"
        with_message = "forbidden"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::NotSchemaMatch {
            message: "This value is forbidden.".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_not_schema_match_without_custom_error_message(
        r#"
        without_message = "forbidden"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::NotSchemaMatch {
            message: "\"not\" schema is matched".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_not_schema_match_valid_value(
        r#"
        with_message = "allowed"
        without_message = "allowed"
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}
