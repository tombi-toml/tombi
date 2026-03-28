use std::path::Path;

use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;

#[derive(Debug, Clone)]
pub(crate) struct PackageLocation {
    pub(crate) pyproject_toml_path: std::path::PathBuf,
    pub(crate) package_name_key_range: tombi_text::Range,
}

impl From<PackageLocation> for Option<tombi_extension::DefinitionLocation> {
    fn from(package_location: PackageLocation) -> Self {
        let Ok(uri) = tombi_uri::Uri::from_file_path(&package_location.pyproject_toml_path) else {
            return None;
        };

        Some(tombi_extension::DefinitionLocation {
            uri,
            range: package_location.package_name_key_range,
        })
    }
}

pub(crate) fn load_pyproject_toml_document_tree(
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<tombi_document_tree::DocumentTree> {
    tombi_extension::load_toml_document_tree(pyproject_toml_path, toml_version)
}

pub(crate) fn find_workspace_pyproject_toml(
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<(
    std::path::PathBuf,
    tombi_ast::Root,
    tombi_document_tree::DocumentTree,
)> {
    tombi_extension::find_ancestor_manifest(
        pyproject_toml_path,
        "pyproject.toml",
        toml_version,
        |tree| tombi_document_tree::dig_keys(tree, &["tool", "uv", "workspace"]).is_some(),
    )
}

pub(crate) fn get_project_name(
    document_tree: &tombi_document_tree::DocumentTree,
) -> Option<&tombi_document_tree::String> {
    match dig_keys(document_tree, &["project", "name"]) {
        Some((_, tombi_document_tree::Value::String(name))) => Some(name),
        _ => None,
    }
}

pub(crate) fn resolve_member_pyproject_toml_path(
    base_pyproject_toml_path: &Path,
    dependency_path: &str,
) -> Option<std::path::PathBuf> {
    tombi_extension::resolve_manifest_path(
        base_pyproject_toml_path,
        Path::new(dependency_path),
        "pyproject.toml",
    )
}
