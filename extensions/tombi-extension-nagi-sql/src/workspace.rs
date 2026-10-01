use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tombi_ast_syntax::AstNode as _;
use tombi_config::TomlVersion;
use tombi_document_tree_syntax::{TryIntoDocumentTree, Value, dig_accessors, dig_keys};
use tombi_extension::SpanConverter;
use tombi_schema_store::{Accessor, matches_accessors};
use tombi_text::EncodingKind;

const CONFIG_PATHS: &[&str] = &["nagi.toml", ".nagi.toml", ".config/nagi.toml"];

pub fn is_nagi_config(uri: &tombi_uri::Uri) -> bool {
    matches!(
        uri.path().rsplit('/').next(),
        Some("nagi.toml" | ".nagi.toml")
    )
}

pub(crate) fn workspace_navigation(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    toml_version: TomlVersion,
    converter: SpanConverter<'_, '_>,
) -> Vec<tombi_extension::Location> {
    let Ok(config_path) = text_document_uri.to_file_path() else {
        return Vec::new();
    };

    if matches_accessors!(accessors, ["workspace", "members"])
        || matches_accessors!(accessors, ["workspace", "members", _])
    {
        return member_config_locations(document_tree, accessors, &config_path);
    }

    if matches_accessors!(accessors, ["sources", _])
        || matches_accessors!(accessors, ["sources", _, "workspace"])
    {
        return workspace_source_location(
            document_tree,
            accessors,
            &config_path,
            toml_version,
            converter,
        )
        .into_iter()
        .collect();
    }

    Vec::new()
}

pub(crate) fn workspace_source_definition_location(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    converter: SpanConverter<'_, '_>,
) -> Option<tombi_extension::Location> {
    if !matches_accessors!(accessors, ["workspace", "sources", _]) {
        return None;
    }

    let source_name = accessors.get(2)?.as_key()?;
    let (source_key, _) = dig_keys(document_tree, &["workspace", "sources", source_name])?;

    Some(converter.location(text_document_uri.clone(), source_key.unquoted_span()))
}

pub(crate) fn workspace_source_reference_locations(
    text_document_uri: &tombi_uri::Uri,
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    toml_version: TomlVersion,
    converter: SpanConverter<'_, '_>,
) -> Vec<tombi_extension::Location> {
    if workspace_source_definition_location(
        text_document_uri,
        workspace_document_tree,
        accessors,
        converter,
    )
    .is_none()
    {
        return Vec::new();
    }

    let Some(source_name) = accessors.get(2).and_then(Accessor::as_key) else {
        return Vec::new();
    };
    let Ok(workspace_config_path) = text_document_uri.to_file_path() else {
        return Vec::new();
    };
    let Some(workspace_root) = config_root(&workspace_config_path) else {
        return Vec::new();
    };
    if find_config_path(workspace_root).as_deref() != Some(workspace_config_path.as_path()) {
        return Vec::new();
    }

    std::iter::once(workspace_config_path.clone())
        .chain(
            config_paths_under(workspace_root)
                .into_iter()
                .filter(|config_path| config_path != &workspace_config_path),
        )
        .filter_map(|config_path| {
            if config_path == workspace_config_path {
                source_reference_location(
                    workspace_document_tree,
                    &config_path,
                    source_name,
                    converter,
                )
            } else {
                load_config(
                    &config_path,
                    toml_version,
                    converter.encoding(),
                    |document_tree, member_converter| {
                        if dig_keys(document_tree, &["workspace"]).is_some()
                            || !matches!(
                                find_workspace_config(
                                    &config_path,
                                    toml_version,
                                    converter.encoding(),
                                    |authority_path, _, _| Some(authority_path),
                                ),
                                Some(authority_path) if authority_path == workspace_config_path
                            )
                        {
                            return None;
                        }
                        source_reference_location(
                            document_tree,
                            &config_path,
                            source_name,
                            member_converter,
                        )
                    },
                )
            }
        })
        .collect()
}

fn config_paths_under(root: &Path) -> BTreeSet<PathBuf> {
    let mut config_paths = BTreeSet::new();
    if let Some(config_path) = find_config_path(root) {
        config_paths.insert(config_path);
    }

    for relative_path in CONFIG_PATHS {
        let pattern = root.join("**").join(relative_path);
        let Ok(candidates) = tombi_fs::glob(&pattern.to_string_lossy()) else {
            continue;
        };
        for candidate in candidates {
            let Some(candidate_root) = config_root(&candidate) else {
                continue;
            };
            if let Some(config_path) = find_config_path(candidate_root) {
                config_paths.insert(config_path);
            }
        }
    }

    config_paths
}

fn source_reference_location(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    config_path: &Path,
    source_name: &str,
    converter: SpanConverter<'_, '_>,
) -> Option<tombi_extension::Location> {
    let (source_key, Value::Table(source)) = dig_keys(document_tree, &["sources", source_name])?
    else {
        return None;
    };
    if !matches!(
        source.get("workspace"),
        Some(Value::Boolean(workspace)) if workspace.value()
    ) {
        return None;
    }

    Some(converter.location(
        tombi_uri::Uri::from_file_path(config_path).ok()?,
        source_key.unquoted_span(),
    ))
}

fn member_config_locations(
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    workspace_config_path: &Path,
) -> Vec<tombi_extension::Location> {
    member_config_paths(workspace_document_tree, accessors, workspace_config_path)
        .into_iter()
        .filter_map(|config_path| {
            Some(tombi_extension::Location {
                uri: tombi_uri::Uri::from_file_path(config_path).ok()?,
                range: None,
            })
        })
        .collect()
}

fn member_config_paths(
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    workspace_config_path: &Path,
) -> BTreeSet<PathBuf> {
    let Some(workspace_root) = config_root(workspace_config_path) else {
        return BTreeSet::new();
    };
    let member_patterns = member_patterns(workspace_document_tree, accessors);
    let excluded_roots = excluded_member_roots(workspace_document_tree, workspace_root);
    let mut config_paths = BTreeSet::new();

    for member_pattern in member_patterns {
        let pattern_path = resolve_path(workspace_root, member_pattern.value());
        let Ok(member_roots) = tombi_fs::glob(&pattern_path.to_string_lossy()) else {
            continue;
        };

        for member_path in member_roots {
            let Some(config_path) = resolve_config_path(&member_path) else {
                continue;
            };
            let config_root = config_root(&config_path).unwrap_or(&config_path);

            if excluded_roots.iter().any(|excluded| {
                member_path == *excluded
                    || member_path.starts_with(excluded)
                    || config_path == *excluded
                    || config_path.starts_with(excluded)
                    || config_root == excluded
                    || config_root.starts_with(excluded)
            }) {
                continue;
            }

            config_paths.insert(config_path);
        }
    }

    config_paths
}

fn workspace_source_location(
    member_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    member_config_path: &Path,
    toml_version: TomlVersion,
    converter: SpanConverter<'_, '_>,
) -> Option<tombi_extension::Location> {
    let member_root = config_root(member_config_path)?;
    if find_config_path(member_root).is_some_and(|path| path != member_config_path) {
        return None;
    }

    let source_name = accessors.get(1)?.as_key()?;
    let (_, Value::Table(source)) = dig_keys(member_document_tree, &["sources", source_name])?
    else {
        return None;
    };
    if !matches!(
        source.get("workspace"),
        Some(Value::Boolean(workspace)) if workspace.value()
    ) {
        return None;
    }

    if dig_keys(member_document_tree, &["workspace"]).is_some() {
        let (source_key, _) =
            dig_keys(member_document_tree, &["workspace", "sources", source_name])?;
        return Some(converter.location(
            tombi_uri::Uri::from_file_path(member_config_path).ok()?,
            source_key.unquoted_span(),
        ));
    }

    find_workspace_config(
        member_config_path,
        toml_version,
        converter.encoding(),
        |workspace_config_path, workspace_document_tree, workspace_converter| {
            let (source_key, _) = dig_keys(
                workspace_document_tree,
                &["workspace", "sources", source_name],
            )?;

            Some(workspace_converter.location(
                tombi_uri::Uri::from_file_path(workspace_config_path).ok()?,
                source_key.unquoted_span(),
            ))
        },
    )
}

fn find_workspace_config<R>(
    member_config_path: &Path,
    toml_version: TomlVersion,
    encoding: EncodingKind,
    f: impl FnOnce(
        PathBuf,
        &tombi_document_tree_syntax::DocumentTree<'_>,
        SpanConverter<'_, '_>,
    ) -> Option<R>,
) -> Option<R> {
    let mut f = Some(f);
    let mut current_dir = config_root(member_config_path)?.parent();

    while let Some(dir) = current_dir {
        if let Some(candidate) = find_config_path(dir)
            && let Some(result) = load_config(
                &candidate,
                toml_version,
                encoding,
                |document_tree, converter| {
                    dig_keys(document_tree, &["workspace"])?;
                    Some(f.take()?(candidate.clone(), document_tree, converter))
                },
            )
        {
            return result;
        }
        current_dir = dir.parent();
    }

    None
}

/// Parses `config_path` and runs `f` on it, since the tree borrows the text read here.
fn load_config<R>(
    config_path: &Path,
    toml_version: TomlVersion,
    encoding: EncodingKind,
    f: impl FnOnce(&tombi_document_tree_syntax::DocumentTree<'_>, SpanConverter<'_, '_>) -> Option<R>,
) -> Option<R> {
    let source = tombi_fs::read_to_string(config_path).ok()?;
    let parsed = tombi_parser::parse(&source);
    let root = parsed.root();
    let decoded = root.decode_strings(toml_version);
    let document_tree = root.try_into_document_tree(toml_version, &decoded).ok()?;
    f(
        &document_tree,
        SpanConverter::new(parsed.line_index(), encoding),
    )
}

fn member_patterns<'a>(
    document_tree: &'a tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
) -> Vec<&'a tombi_document_tree_syntax::String<'a>> {
    if matches_accessors!(accessors, ["workspace", "members", _]) {
        return match dig_accessors(document_tree, accessors) {
            Some((_, Value::String(pattern))) => vec![pattern],
            _ => Vec::new(),
        };
    }

    match dig_keys(document_tree, &["workspace", "members"]) {
        Some((_, Value::Array(members))) => members
            .iter()
            .filter_map(|member| match member {
                Value::String(pattern) => Some(pattern),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn excluded_member_roots(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    workspace_root: &Path,
) -> Vec<PathBuf> {
    let Some((_, Value::Array(excludes))) = dig_keys(document_tree, &["workspace", "exclude"])
    else {
        return Vec::new();
    };

    excludes
        .iter()
        .filter_map(|exclude| match exclude {
            Value::String(pattern) => Some(resolve_path(workspace_root, pattern.value())),
            _ => None,
        })
        .filter_map(|pattern| tombi_fs::glob(&pattern.to_string_lossy()).ok())
        .flatten()
        .collect()
}

fn resolve_path(root: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn find_config_path(root: &Path) -> Option<PathBuf> {
    CONFIG_PATHS
        .iter()
        .map(|relative_path| root.join(relative_path))
        .find(|path| tombi_fs::is_file(path))
}

fn resolve_config_path(path: &Path) -> Option<PathBuf> {
    if matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("nagi.toml" | ".nagi.toml")
    ) {
        return tombi_fs::is_file(path).then(|| path.to_path_buf());
    }

    tombi_fs::is_dir(path)
        .then(|| find_config_path(path))
        .flatten()
}

pub(crate) fn config_root(config_path: &Path) -> Option<&Path> {
    let parent = config_path.parent()?;
    if config_path.ends_with(Path::new(".config/nagi.toml")) {
        parent.parent().or(Some(parent))
    } else {
        Some(parent)
    }
}
