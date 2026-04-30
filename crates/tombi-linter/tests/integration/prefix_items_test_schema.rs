use tombi_linter::test_lint;
use tombi_test_lib::prefix_items_test_schema_path;

test_lint! {
    #[test]
    fn test_prefix_items_valid(
        r#"
        point = [1.0, 2.0]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_prefix_items_type_mismatch(
        r#"
        point = ["hello", 2.0]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::Float,
            actual: tombi_document_tree::ValueType::String,
        },
    ])
}

test_lint! {
    #[test]
    fn test_prefix_items_overflow_rejected(
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

test_lint! {
    #[test]
    fn test_prefix_items_with_overflow_schema_valid(
        r#"
        extensible = [1, "hello", "extra1", "extra2"]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_prefix_items_with_overflow_schema_invalid(
        r#"
        extensible = [1, "hello", 42]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::String,
            actual: tombi_document_tree::ValueType::Integer,
        },
    ])
}

test_lint! {
    #[test]
    fn test_prefix_items_open_with_extra(
        r#"
        open_tuple = ["hello", 42, true, 3.14]
        "#,
        SchemaPath(prefix_items_test_schema_path()),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_prefix_items_subschema_root_matches_exact_tuple_index(
        r#"
        tuple = [1, "hello"]
        "#,
        Config({
            let mut config = tombi_config::Config::default();
            config.schema = Some(tombi_config::SchemaOverviewOptions::default());
            config.schemas = Some(vec![tombi_config::SchemaItem::Sub(tombi_config::SubSchema {
                root: "tuple[1]".to_string(),
                path: "schemas/prefix-items-test.schema.json#/properties/extensible/prefixItems/1"
                    .to_string(),
                include: vec!["test.toml".to_string()],
                exclude: None,
                format: None,
                lint: None,
                overrides: None,
            })]);
            config
        }),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_exact_index_string_subschema_does_not_apply_to_first_item(
        r#"
        items = [42, "scoped"]
        "#,
        Config({
            let mut config = tombi_config::Config::default();
            config.schema = Some(tombi_config::SchemaOverviewOptions::default());
            config.schemas = Some(vec![tombi_config::SchemaItem::Sub(tombi_config::SubSchema {
                root: "items[1]".to_string(),
                path: tombi_schema_store::SchemaUri::from_file_path(
                    tombi_test_lib::project_root_path().join("schemas/exact-index-string-test.schema.json"),
                )
                .unwrap()
                .to_string(),
                include: vec!["test.toml".to_string()],
                exclude: None,
                format: None,
                lint: None,
                overrides: None,
            })]);
            config
        }),
    ) -> Ok(_)
}

test_lint! {
    #[test]
    fn test_exact_index_string_subschema_applies_to_second_item(
        r#"
        items = [42, 7]
        "#,
        Config({
            let mut config = tombi_config::Config::default();
            config.schema = Some(tombi_config::SchemaOverviewOptions::default());
            config.schemas = Some(vec![tombi_config::SchemaItem::Sub(tombi_config::SubSchema {
                root: "items[1]".to_string(),
                path: tombi_schema_store::SchemaUri::from_file_path(
                    tombi_test_lib::project_root_path().join("schemas/exact-index-string-test.schema.json"),
                )
                .unwrap()
                .to_string(),
                include: vec!["test.toml".to_string()],
                exclude: None,
                format: None,
                lint: None,
                overrides: None,
            })]);
            config
        }),
    ) -> Err([
        tombi_validator::DiagnosticKind::TypeMismatch {
            expected: tombi_schema_store::ValueType::String,
            actual: tombi_document_tree::ValueType::Integer,
        },
    ])
}
