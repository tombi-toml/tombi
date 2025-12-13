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
        .join(tombi_uri::schemastore_hostname!())
        .join("tombi.json")
}

pub fn cargo_schema_path() -> PathBuf {
    project_root_path()
        .join(tombi_uri::schemastore_hostname!())
        .join("cargo.json")
}

pub fn pyproject_schema_path() -> PathBuf {
    project_root_path()
        .join(tombi_uri::schemastore_hostname!())
        .join("pyproject.json")
}

pub fn type_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("type-test.schema.json")
}

pub fn untagged_union_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("untagged-union.schema.json")
}
