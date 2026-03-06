use tombi_linter::test_lint;
use tombi_test_lib::array_const_enum_test_schema_path;

test_lint! {
    #[test]
    fn test_array_const_valid(
        r#"
        fixed_list = [1, 2, 3]
        "#,
        SchemaPath(array_const_enum_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_array_const_invalid(
        r#"
        fixed_list = [1, 2, 4]
        "#,
        SchemaPath(array_const_enum_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::Const {
            expected: "[1, 2, 3]".to_string(),
            actual: "[1, 2, 4]".to_string(),
        },
    ])
}

test_lint! {
    #[test]
    fn test_array_enum_valid(
        r#"
        color = [0, 255, 0]
        "#,
        SchemaPath(array_const_enum_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_array_enum_invalid(
        r#"
        color = [128, 128, 128]
        "#,
        SchemaPath(array_const_enum_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::Enum {
            expected: vec![
                "[255, 0, 0]".to_string(),
                "[0, 255, 0]".to_string(),
                "[0, 0, 255]".to_string(),
            ],
            actual: "[128, 128, 128]".to_string(),
        },
    ])
}
