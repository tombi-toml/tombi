use std::path::PathBuf;

pub fn project_root_path() -> PathBuf {
    let dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned());

    PathBuf::from(dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

pub fn tombi_schema_path() -> PathBuf {
    project_root_path()
        .join("json.schemastore.org")
        .join("tombi.json")
}

pub fn cargo_schema_path() -> PathBuf {
    project_root_path()
        .join("json.schemastore.org")
        .join("cargo.json")
}

pub fn pyproject_schema_path() -> PathBuf {
    project_root_path()
        .join("json.schemastore.org")
        .join("pyproject.json")
}

pub fn type_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("type-test.schema.json")
}

pub fn x_tombi_table_keys_order_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("x-tombi-table-keys-order.schema.json")
}
