use std::path::Path;

use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{TryIntoDocumentTree, dig_keys};

#[derive(Debug, Clone)]
pub(crate) struct PackageLocation {
    pub(crate) pyproject_toml_path: std::path::PathBuf,
    pub(crate) package_name_key_range: tombi_text::Range,
}

impl From<PackageLocation> for Option<tombi_extension::Location> {
    fn from(package_location: PackageLocation) -> Self {
        let Ok(uri) = tombi_uri::Uri::from_file_path(&package_location.pyproject_toml_path) else {
            return None;
        };

        Some(tombi_extension::Location {
            uri,
            range: package_location.package_name_key_range,
        })
    }
}

pub(crate) fn load_pyproject_toml_document_tree(
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<tombi_document_tree::DocumentTree> {
    let (_, document_tree) = load_pyproject_toml(pyproject_toml_path, toml_version)?;
    Some(document_tree)
}

pub(crate) fn find_workspace_pyproject_toml(
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<(
    std::path::PathBuf,
    tombi_ast::Root,
    tombi_document_tree::DocumentTree,
)> {
    if let Some((root, document_tree)) = load_pyproject_toml(pyproject_toml_path, toml_version)
        && tombi_document_tree::dig_keys(&document_tree, &["tool", "uv", "workspace"]).is_some()
    {
        return Some((pyproject_toml_path.to_path_buf(), root, document_tree));
    }

    let (workspace_pyproject_toml_path, (root, document_tree)) =
        tombi_extension_manifest::find_ancestor_manifest(
            pyproject_toml_path,
            "pyproject.toml",
            |path| load_pyproject_toml(path, toml_version),
            |(_, tree)| tombi_document_tree::dig_keys(tree, &["tool", "uv", "workspace"]).is_some(),
        )?;

    Some((workspace_pyproject_toml_path, root, document_tree))
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
    tombi_extension_manifest::resolve_manifest_path(
        base_pyproject_toml_path,
        Path::new(dependency_path),
        "pyproject.toml",
    )
}

pub(crate) fn resolve_relative_path_uri(
    base_pyproject_toml_path: &Path,
    relative_path: &Path,
) -> Option<tombi_uri::Uri> {
    let resolved_path = if relative_path.is_absolute() {
        relative_path.to_path_buf()
    } else {
        base_pyproject_toml_path.parent()?.join(relative_path)
    };

    resolved_path.exists().then_some(())?;

    tombi_uri::Uri::from_file_path(&resolved_path).ok()
}

fn load_pyproject_toml(
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<(tombi_ast::Root, tombi_document_tree::DocumentTree)> {
    let toml_text = std::fs::read_to_string(pyproject_toml_path).ok()?;
    let root = tombi_ast::Root::cast(tombi_parser::parse(&toml_text).into_syntax_node())?;

    Some((
        root.clone(),
        root.try_into_document_tree(toml_version).ok()?,
    ))
}
