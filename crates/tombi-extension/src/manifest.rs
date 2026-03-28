use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{DocumentTree, TryIntoDocumentTree};

pub type LoadedToml = (tombi_ast::Root, DocumentTree);

pub fn load_toml(toml_path: &Path, toml_version: TomlVersion) -> Option<LoadedToml> {
    let toml_text = std::fs::read_to_string(toml_path).ok()?;
    let root = tombi_ast::Root::cast(tombi_parser::parse(&toml_text).into_syntax_node())?;

    Some((
        root.clone(),
        root.try_into_document_tree(toml_version).ok()?,
    ))
}

pub fn load_toml_document_tree(
    toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<DocumentTree> {
    let (_, document_tree) = load_toml(toml_path, toml_version)?;
    Some(document_tree)
}

pub fn find_ancestor_manifest(
    toml_path: &Path,
    manifest_file_name: &str,
    toml_version: TomlVersion,
    is_workspace: impl Fn(&DocumentTree) -> bool,
) -> Option<(PathBuf, tombi_ast::Root, DocumentTree)> {
    let mut current_dir = toml_path.parent()?;

    while let Some(target_dir) = current_dir.parent() {
        current_dir = target_dir;
        let workspace_toml_path = current_dir.join(manifest_file_name);

        let Some((root, document_tree)) = load_toml(&workspace_toml_path, toml_version) else {
            continue;
        };

        if is_workspace(&document_tree) {
            return Some((workspace_toml_path, root, document_tree));
        }
    }

    None
}

pub fn resolve_manifest_path(
    base_manifest_path: &Path,
    candidate_path: &Path,
    manifest_file_name: &str,
) -> Option<PathBuf> {
    let resolved_path = if candidate_path.is_absolute() {
        candidate_path.to_path_buf()
    } else {
        base_manifest_path.parent()?.join(candidate_path)
    };

    let toml_path = if resolved_path.file_name() == Some(OsStr::new(manifest_file_name)) {
        resolved_path
    } else {
        resolved_path.join(manifest_file_name)
    };

    toml_path.is_file().then_some(toml_path)
}

pub fn resolve_relative_file_uri(
    base_manifest_path: &Path,
    relative_path: &Path,
) -> Option<tombi_uri::Uri> {
    let resolved_path = if relative_path.is_absolute() {
        relative_path.to_path_buf()
    } else {
        base_manifest_path.parent()?.join(relative_path)
    };

    if !resolved_path.is_file() {
        return None;
    }

    tombi_uri::Uri::from_file_path(&resolved_path).ok()
}

pub fn find_member_manifest_paths<'a>(
    member_patterns: &'a [&'a tombi_document_tree::String],
    exclude_patterns: &'a [&'a tombi_document_tree::String],
    workspace_dir_path: &'a Path,
    manifest_file_name: &'static str,
) -> impl Iterator<Item = (&'a tombi_document_tree::String, PathBuf)> + 'a {
    let exclude_patterns = exclude_patterns
        .iter()
        .filter_map(|pattern| glob::Pattern::new(pattern.value()).ok())
        .collect::<Vec<_>>();

    member_patterns
        .iter()
        .filter_map(move |&member_pattern| {
            let mut toml_paths = vec![];

            let mut member_pattern_path = Path::new(member_pattern.value()).to_path_buf();
            if !member_pattern_path.is_absolute() {
                member_pattern_path = workspace_dir_path.join(member_pattern_path);
            }

            let mut candidate_paths = match glob::glob(&member_pattern_path.to_string_lossy()) {
                Ok(paths) => paths,
                Err(_) => return None,
            };

            while let Some(Ok(candidate_path)) = candidate_paths.next() {
                if !candidate_path.is_dir() {
                    continue;
                }

                let toml_path = candidate_path.join(manifest_file_name);
                if !toml_path.is_file() {
                    continue;
                }

                let is_excluded = exclude_patterns
                    .iter()
                    .any(|exclude_pattern| exclude_pattern.matches(&toml_path.to_string_lossy()));

                if !is_excluded {
                    toml_paths.push((member_pattern, toml_path));
                }
            }

            (!toml_paths.is_empty()).then_some(toml_paths)
        })
        .flatten()
}
