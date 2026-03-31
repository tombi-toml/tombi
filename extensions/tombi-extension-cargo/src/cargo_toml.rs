use std::path::Path;

use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::TryIntoDocumentTree;

#[derive(Debug, Clone)]
pub(crate) struct CrateLocation {
    pub(crate) cargo_toml_path: std::path::PathBuf,
    pub(crate) package_name_key_range: tombi_text::Range,
}

impl From<CrateLocation> for Option<tombi_extension::DefinitionLocation> {
    fn from(crate_location: CrateLocation) -> Self {
        let Ok(uri) = tombi_uri::Uri::from_file_path(&crate_location.cargo_toml_path) else {
            return None;
        };

        Some(tombi_extension::DefinitionLocation {
            uri,
            range: crate_location.package_name_key_range,
        })
    }
}

pub(crate) fn load_cargo_toml(
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<(tombi_ast::Root, tombi_document_tree::DocumentTree)> {
    let toml_text = std::fs::read_to_string(cargo_toml_path).ok()?;
    let root = tombi_ast::Root::cast(tombi_parser::parse(&toml_text).into_syntax_node())?;

    Some((
        root.clone(),
        root.try_into_document_tree(toml_version).ok()?,
    ))
}

pub(crate) fn find_cargo_toml(
    cargo_toml_path: &Path,
    crate_path: &Path,
    toml_version: TomlVersion,
) -> Option<(
    std::path::PathBuf,
    tombi_ast::Root,
    tombi_document_tree::DocumentTree,
)> {
    let crate_cargo_toml_path =
        tombi_extension_manifest::resolve_manifest_path(cargo_toml_path, crate_path, "Cargo.toml")?;
    let canonicalized_path = crate_cargo_toml_path.canonicalize().ok()?;
    let (root, document_tree) = load_cargo_toml(&canonicalized_path, toml_version)?;

    Some((canonicalized_path, root, document_tree))
}

pub(crate) fn dependency_package_name<'a>(
    dependency_key: &'a str,
    dependency_value: &'a tombi_document_tree::Value,
) -> &'a str {
    match dependency_value {
        tombi_document_tree::Value::Table(table) => match table.get("package") {
            Some(tombi_document_tree::Value::String(package)) => package.value(),
            _ => dependency_key,
        },
        _ => dependency_key,
    }
}

pub(crate) fn get_uri_relative_to_cargo_toml(
    relative_path: &Path,
    cargo_toml_path: &Path,
) -> Option<tombi_uri::Uri> {
    tombi_extension_manifest::resolve_relative_file_uri(cargo_toml_path, relative_path)
}
