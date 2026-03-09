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

pub fn recursive_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("recursive-schema.schema.json")
}

pub fn if_then_else_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("if-then-else-test.schema.json")
}

pub fn contains_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("contains-test.schema.json")
}

pub fn dependencies_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("dependencies-test.schema.json")
}

pub fn dependencies_strict_mode_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("dependencies-strict-mode-test.schema.json")
}

pub fn tuple_items_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("tuple-items-test.schema.json")
}

pub fn prefix_items_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("prefix-items-test.schema.json")
}

pub fn table_const_enum_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("table-const-enum-test.schema.json")
}

pub fn array_const_enum_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("array-const-enum-test.schema.json")
}

pub fn string_format_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("string-format-test.schema.json")
}

pub fn dependent_required_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("dependent-required-test.schema.json")
}

pub fn dependent_schemas_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("dependent-schemas-test.schema.json")
}

pub fn min_max_contains_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("min-max-contains-test.schema.json")
}

pub fn anchor_dynamic_ref_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("anchor-dynamic-ref-test.schema.json")
}

pub fn recursive_anchor_ref_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("recursive-anchor-ref-test.schema.json")
}

pub fn format_annotation_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("format-annotation-test.schema.json")
}

pub fn format_assertion_vocab_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("format-assertion-vocab-test.schema.json")
}

pub fn unevaluated_items_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("unevaluated-items-test.schema.json")
}

pub fn unevaluated_properties_test_schema_path() -> PathBuf {
    project_root_path()
        .join("schemas")
        .join("unevaluated-properties-test.schema.json")
}
