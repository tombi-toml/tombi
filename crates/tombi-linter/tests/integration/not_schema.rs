use tombi_linter::test_lint;
use tombi_schema_store::SchemaUri;

fn schema_path() -> std::path::PathBuf {
    tombi_test_lib::project_root_path()
        .join("schemas")
        .join("not-schema-test.schema.json")
}

test_lint! {
    #[test]
    fn test_unresolved_ref_reports_error(
        r#"
        value = "foo"
        "#,
        SchemaPath(schema_path()),
    ) -> Err([
        tombi_schema_store::Error::InvalidJsonPointer {
            pointer: "#/$defs/missing".to_string(),
            schema_uri: SchemaUri::from_file_path(schema_path()).unwrap(),
        }
    ])
}
