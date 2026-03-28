use std::path::Path;

use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{TryIntoDocumentTree, dig_keys};

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

pub(crate) fn find_workspace_cargo_toml(
    cargo_toml_path: &Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
) -> Option<(
    std::path::PathBuf,
    tombi_ast::Root,
    tombi_document_tree::DocumentTree,
)> {
    if let Some(workspace_path) = workspace_path {
        let workspace_cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
            cargo_toml_path,
            Path::new(workspace_path),
            "Cargo.toml",
        )?;
        let canonicalized_path = workspace_cargo_toml_path.canonicalize().ok()?;
        let (root, document_tree) = load_cargo_toml(&canonicalized_path, toml_version)?;

        return document_tree.contains_key("workspace").then_some((
            canonicalized_path,
            root,
            document_tree,
        ));
    }

    let (workspace_cargo_toml_path, (root, document_tree)) =
        tombi_extension_manifest::find_ancestor_manifest(
            cargo_toml_path,
            "Cargo.toml",
            |path| load_cargo_toml(path, toml_version),
            |(_, tree)| tree.contains_key("workspace"),
        )?;

    Some((workspace_cargo_toml_path, root, document_tree))
}

pub(crate) fn find_path_crate_cargo_toml(
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

/// Get the workspace path from Cargo.toml
///
/// See: https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field
#[inline]
pub(crate) fn get_workspace_path(
    document_tree: &tombi_document_tree::DocumentTree,
) -> Option<&str> {
    dig_keys(document_tree, &["package", "workspace"]).and_then(|(_, workspace)| {
        if let tombi_document_tree::Value::String(workspace_path) = workspace {
            Some(workspace_path.value())
        } else {
            None
        }
    })
}

pub(crate) fn get_uri_relative_to_cargo_toml(
    relative_path: &Path,
    cargo_toml_path: &Path,
) -> Option<tombi_uri::Uri> {
    tombi_extension_manifest::resolve_relative_file_uri(cargo_toml_path, relative_path)
}
