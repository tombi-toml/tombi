use std::path::Path;

use tombi_ast_syntax::AstNode as _;
use tombi_config::TomlVersion;
use tombi_document_tree_syntax::{TryIntoDocumentTree, dig_keys};
use tombi_extension::SpanConverter;
use tombi_text::EncodingKind;

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
            range: Some(package_location.package_name_key_range),
        })
    }
}

pub(crate) fn find_workspace_pyproject_toml<R>(
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
    encoding: EncodingKind,
    f: impl FnOnce(
        std::path::PathBuf,
        tombi_ast_syntax::Root<'_>,
        &tombi_document_tree_syntax::DocumentTree<'_>,
        SpanConverter<'_, '_>,
    ) -> R,
) -> Option<R> {
    let mut f = Some(f);
    let mut try_load = |path: &Path| {
        with_pyproject_toml(
            path,
            toml_version,
            encoding,
            |root, document_tree, converter| {
                dig_keys(document_tree, &["tool", "uv", "workspace"])?;
                Some(f.take()?(
                    path.to_path_buf(),
                    root,
                    document_tree,
                    converter,
                ))
            },
        )
        .flatten()
    };

    if let Some(result) = try_load(pyproject_toml_path) {
        return Some(result);
    }

    let mut current_dir = pyproject_toml_path.parent()?;
    while let Some(target_dir) = current_dir.parent() {
        current_dir = target_dir;
        let workspace_pyproject_toml_path = current_dir.join("pyproject.toml");
        if !tombi_fs::is_file(&workspace_pyproject_toml_path) {
            continue;
        }
        if let Some(result) = try_load(&workspace_pyproject_toml_path) {
            return Some(result);
        }
    }

    None
}

pub(crate) fn get_project_name<'a, 't>(
    document_tree: &'a tombi_document_tree_syntax::DocumentTree<'t>,
) -> Option<&'a tombi_document_tree_syntax::String<'t>> {
    match dig_keys(document_tree, &["project", "name"]) {
        Some((_, tombi_document_tree_syntax::Value::String(name))) => Some(name),
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

/// Parses `pyproject_toml_path` and runs `f` on it, since the tree borrows the text read here.
///
/// Returns `None` when the file cannot be read or is not a valid TOML document.
pub(crate) fn with_pyproject_toml<R>(
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
    encoding: EncodingKind,
    f: impl FnOnce(
        tombi_ast_syntax::Root<'_>,
        &tombi_document_tree_syntax::DocumentTree<'_>,
        SpanConverter<'_, '_>,
    ) -> R,
) -> Option<R> {
    let toml_text = tombi_fs::read_to_string(pyproject_toml_path).ok()?;
    let parsed = tombi_parser::parse(&toml_text);
    let root = parsed.root();
    let decoded = root.decode_strings(toml_version);
    let document_tree = root.try_into_document_tree(toml_version, &decoded).ok()?;

    Some(f(
        root,
        &document_tree,
        SpanConverter::new(parsed.line_index(), encoding),
    ))
}
