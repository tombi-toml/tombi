use std::path::Path;

use tombi_ast_syntax::AstNode as _;
use tombi_config::TomlVersion;
use tombi_document_tree_syntax::TryIntoDocumentTree;

#[derive(Debug, Clone)]
pub(crate) struct CrateLocation {
    pub(crate) cargo_toml_path: std::path::PathBuf,
    /// The range of the package name key in the manifest, in the unit of the client's encoding.
    pub(crate) package_name_key_range: tombi_text::Range,
}

impl From<CrateLocation> for Option<tombi_extension::Location> {
    fn from(crate_location: CrateLocation) -> Self {
        let Ok(uri) = tombi_uri::Uri::from_file_path(&crate_location.cargo_toml_path) else {
            return None;
        };

        Some(tombi_extension::Location {
            uri,
            range: Some(crate_location.package_name_key_range),
        })
    }
}

/// Parses `toml_text` and runs `f` on the document tree and the line index of the text.
///
/// The tree borrows `toml_text`, so `f` must return owned data.
pub(crate) fn with_cargo_toml_text<R>(
    toml_text: &str,
    toml_version: TomlVersion,
    f: impl FnOnce(&tombi_document_tree_syntax::DocumentTree<'_>, &tombi_text::LineIndex<'_>) -> R,
) -> Option<R> {
    with_cargo_toml_text_and_root(toml_text, toml_version, |_, document_tree, line_index| {
        f(document_tree, line_index)
    })
}

/// Like [`with_cargo_toml_text`], and also gives the AST root of the text to `f`.
pub(crate) fn with_cargo_toml_text_and_root<R>(
    toml_text: &str,
    toml_version: TomlVersion,
    f: impl FnOnce(
        tombi_ast_syntax::Root<'_>,
        &tombi_document_tree_syntax::DocumentTree<'_>,
        &tombi_text::LineIndex<'_>,
    ) -> R,
) -> Option<R> {
    let parsed = tombi_parser::parse(toml_text);
    let root = parsed.root();
    let decoded = root.decode_strings(toml_version);
    let document_tree = root.try_into_document_tree(toml_version, &decoded).ok()?;

    Some(f(root, &document_tree, parsed.line_index()))
}

/// Reads and parses `cargo_toml_path`, then runs `f` on its document tree and line index.
pub(crate) fn load_cargo_toml<R>(
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
    f: impl FnOnce(&tombi_document_tree_syntax::DocumentTree<'_>, &tombi_text::LineIndex<'_>) -> R,
) -> Option<R> {
    let toml_text = tombi_fs::read_to_string(cargo_toml_path).ok()?;

    with_cargo_toml_text(&toml_text, toml_version, f)
}

/// Like [`load_cargo_toml`], and also gives the AST root to `f`.
pub(crate) fn load_cargo_toml_with_root<R>(
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
    f: impl FnOnce(
        tombi_ast_syntax::Root<'_>,
        &tombi_document_tree_syntax::DocumentTree<'_>,
        &tombi_text::LineIndex<'_>,
    ) -> R,
) -> Option<R> {
    let toml_text = tombi_fs::read_to_string(cargo_toml_path).ok()?;

    with_cargo_toml_text_and_root(&toml_text, toml_version, f)
}

/// Resolves the `Cargo.toml` of `crate_path` and runs `f` on its canonicalized path,
/// document tree and line index.
pub(crate) fn find_cargo_toml<R>(
    cargo_toml_path: &Path,
    crate_path: &Path,
    toml_version: TomlVersion,
    f: impl FnOnce(
        &Path,
        &tombi_document_tree_syntax::DocumentTree<'_>,
        &tombi_text::LineIndex<'_>,
    ) -> R,
) -> Option<R> {
    let crate_cargo_toml_path =
        tombi_extension_manifest::resolve_manifest_path(cargo_toml_path, crate_path, "Cargo.toml")?;
    let canonicalized_path = tombi_fs::canonicalize(&crate_cargo_toml_path).ok()?;

    load_cargo_toml(
        &canonicalized_path,
        toml_version,
        |document_tree, line_index| f(&canonicalized_path, document_tree, line_index),
    )
}

pub(crate) fn dependency_package_name<'a>(
    dependency_key: &'a str,
    dependency_value: &'a tombi_document_tree_syntax::Value<'_>,
) -> &'a str {
    match dependency_value {
        tombi_document_tree_syntax::Value::Table(table) => match table.get("package") {
            Some(tombi_document_tree_syntax::Value::String(package)) => package.value(),
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
