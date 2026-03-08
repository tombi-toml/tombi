use tombi_linter::test_lint;
use tombi_test_lib::{format_annotation_test_schema_path, format_assertion_vocab_test_schema_path};
use tombi_x_keyword::StringFormat;

// --- Draft 2019-09: `format` is annotation-only by default ---
// Invalid format values should NOT produce errors.

test_lint! {
    #[test]
    fn test_format_annotation_invalid_ipv4_accepted(
        r#"
        ipv4_addr = "not-an-ipv4"
        "#,
        SchemaPath(format_annotation_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_format_annotation_invalid_email_accepted(
        r#"
        email_addr = "not-an-email"
        "#,
        SchemaPath(format_annotation_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_format_annotation_valid_ipv4_accepted(
        r#"
        ipv4_addr = "192.168.1.1"
        "#,
        SchemaPath(format_annotation_test_schema_path()),
    ) -> Ok(_)
}

// --- Draft 2020-12 with $vocabulary format-assertion enabled ---
// Invalid format values SHOULD produce errors.

test_lint! {
    #[test]
    fn test_format_assertion_vocab_invalid_ipv4_rejected(
        r#"
        ipv4_addr = "not-an-ipv4"
        "#,
        SchemaPath(format_assertion_vocab_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::StringFormat {
            format: StringFormat::Ipv4,
            actual: "\"not-an-ipv4\"".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_format_assertion_vocab_valid_ipv4_accepted(
        r#"
        ipv4_addr = "192.168.1.1"
        "#,
        SchemaPath(format_assertion_vocab_test_schema_path()),
    ) -> Ok(_)
}
