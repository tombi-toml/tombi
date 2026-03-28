use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub fn find_ancestor_manifest<T>(
    toml_path: &Path,
    manifest_file_name: &str,
    load_manifest: impl Fn(&Path) -> Option<T>,
    is_workspace: impl Fn(&T) -> bool,
) -> Option<(PathBuf, T)> {
    let mut current_dir = toml_path.parent()?;

    while let Some(target_dir) = current_dir.parent() {
        current_dir = target_dir;
        let workspace_toml_path = current_dir.join(manifest_file_name);

        if !workspace_toml_path.is_file() {
            continue;
        }

        let Some(manifest) = load_manifest(&workspace_toml_path) else {
            continue;
        };

        if is_workspace(&manifest) {
            return Some((workspace_toml_path, manifest));
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
