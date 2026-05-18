use std::path::PathBuf;

use tombi_linter::test_lint;
use tombi_test_lib::project_root_path;

fn schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("issue-1895-rustfmt-like.schema.json")
}

test_lint! {
    #[test]
    fn annotation_only_property_in_properties_is_allowed_in_strict_mode(
        r#"
        max_width = 120
        ignore = ["*_capnp.rs"]
        "#,
        SchemaPath(schema_path()),
    ) -> Ok(_)
}
