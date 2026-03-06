use std::str::FromStr;
use tombi_linter::test_lint;
use tombi_schema_store::SchemaUri;

test_lint! {
    #[test]
    // Ref: https://github.com/tombi-toml/tombi/issues/1031
    fn test_error_report_case1(
        r#"
        [job]
        name = "foo"
        prod.cpu = 10
        prod.autoscale = { min = 10, max = 20 }
        "#,
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_warning_empty(
        r#"
        "" = 1
        "#,
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyEmpty
    ])
}

test_lint! {
    #[test]
    fn test_empty_document_with_dangling_value_comment_directive(
        r#"
        # tombi: format.rules.table-keys-order = "descending"
        "#,
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_key_value_with_dangling_value_comment_directive(
        r#"
        # tombi: format.rules.table-keys-order = "descending"

        key = "value"
        "#,
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_table_with_dangling_value_comment_directive(
        r#"
        # tombi: format.rules.table-keys-order = "descending"

        [aaa]
        "#,
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_table_warning_empty(
        r#"
        [aaa]
        "" = 1
        "#,
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyEmpty
    ])
}

test_lint! {
    #[test]
    fn test_array_of_table_warning_empty(
        r#"
        [[aaa]]
        "" = 1
        "#,
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyEmpty
    ])
}

test_lint! {
    #[test]
    fn test_inline_table_warning_empty(
        r#"
        key = { "" = 1 }
        "#,
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyEmpty
    ])
}

test_lint! {
    #[test]
    fn test_nested_inline_table_warning_empty(
        r#"
        key = { key2 = { "" = 1 } }
        "#,
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyEmpty
    ])
}

test_lint! {
    #[test]
    fn test_table_inline_table_warning_empty(
        r#"
        [array]
        key = { "" = 1 }
        "#,
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyEmpty
    ])
}

test_lint! {
    #[test]
    fn test_array_of_table_inline_table_warning_empty(
        r#"
        [[array]]
        key = { "" = 1 }
        "#,
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyEmpty
    ])
}

test_lint! {
    #[test]
    fn test_dotted_keys_out_of_order(
        r#"
        apple.type = "fruit"
        orange.type = "fruit"

        apple.skin = "thin"
        orange.skin = "thick"

        apple.color = "red"
        orange.color = "orange"
        "#,
    ) -> Err([
        tombi_linter::DiagnosticKind::DottedKeysOutOfOrder,
        tombi_linter::DiagnosticKind::DottedKeysOutOfOrder,
        tombi_linter::DiagnosticKind::DottedKeysOutOfOrder,
        tombi_linter::DiagnosticKind::DottedKeysOutOfOrder,
        tombi_linter::DiagnosticKind::DottedKeysOutOfOrder,
        tombi_linter::DiagnosticKind::DottedKeysOutOfOrder
    ])
}

test_lint! {
    #[test]
    fn test_dotted_keys_out_of_order_with_comment_directive_table_keys_order_disabled_eq_true(
        r#"
        # tombi: format.rules.table-keys-order.disabled = true

        apple.type = "fruit"
        orange.type = "fruit"

        apple.skin = "thin"
        orange.skin = "thick"

        apple.color = "red"
        orange.color = "orange"
        "#,
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_schema_uri(
        r#"
        #:schema https://www.schemastore.org/tombi.json
        "#,
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_schema_file(
        r#"
        #:schema ./www.schemastore.org/tombi.json
        "#,
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_file_schema_does_not_exist_url(
        r#"
        #:schema https://does-not-exist.co.jp
        "#,
    ) -> Err([
        tombi_schema_store::Error::SchemaFetchFailed{
            schema_uri: SchemaUri::from_str("https://does-not-exist.co.jp").unwrap(),
            reason: "error sending request for url (https://does-not-exist.co.jp/)".to_string(),
        }
    ])
}

test_lint! {
    #[test]
    fn test_file_schema_does_not_exist_file(
        r#"
        #:schema does-not-exist.schema.json
        "#,
    ) -> Err([
        tombi_schema_store::Error::SchemaFileNotFound{
            schema_path: tombi_test_lib::project_root_path().join("does-not-exist.schema.json"),
        }
    ])
}

test_lint! {
    #[test]
    fn test_file_schema_relative_does_not_exist_file(
        r#"
        #:schema ./does-not-exist.schema.json
        "#,
    ) -> Err([
        tombi_schema_store::Error::SchemaFileNotFound{
            schema_path: tombi_test_lib::project_root_path().join("does-not-exist.schema.json"),
        }
    ])
}

test_lint! {
    #[test]
    fn test_file_schema_parent_does_not_exist_file(
        r#"
        #:schema ../does-not-exist.schema.json
        "#,
    ) -> Err([
        tombi_schema_store::Error::SchemaFileNotFound{
            schema_path: tombi_test_lib::project_root_path().join("../does-not-exist.schema.json"),
        }
    ])
}

test_lint! {
    #[test]
    fn test_tombi_document_comment_directive_lint_not_exist_eq_true(
        r#"
        #:tombi lint.not-exist = true
        "#,
    ) -> Err([
        tombi_validator::DiagnosticKind::KeyNotAllowed { key: "not-exist".to_string() }
    ])
}
