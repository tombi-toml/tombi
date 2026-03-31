use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use itertools::Itertools;
use tokio::sync::RwLock;
use tombi_config::TomlVersion;
use tombi_document_tree::{ValueImpl, dig_accessors};
use tombi_extension::file_cache_version;
use tombi_hashmap::HashMap;
use tombi_schema_store::matches_accessors;

use crate::{CrateLocation, find_cargo_toml, get_uri_relative_to_cargo_toml, load_cargo_toml};

const MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES: usize = 128;

#[derive(Clone)]
struct CachedCargoToml {
    version: Option<u64>,
    document_tree: tombi_document_tree::DocumentTree,
}

#[derive(Clone)]
struct CachedWorkspaceCargoToml {
    version: Option<u64>,
    workspace_cargo_toml_path: Option<PathBuf>,
}

static DID_OPEN_CARGO_TOML_CACHE: LazyLock<RwLock<HashMap<PathBuf, CachedCargoToml>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
static DID_OPEN_WORKSPACE_CARGO_TOML_CACHE: LazyLock<
    RwLock<HashMap<PathBuf, CachedWorkspaceCargoToml>>,
> = LazyLock::new(|| RwLock::new(HashMap::new()));

fn canonicalize_or_original(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn insert_cargo_toml(
    cache: &mut HashMap<PathBuf, CachedCargoToml>,
    path: PathBuf,
    value: CachedCargoToml,
) {
    if !cache.contains_key(&path)
        && cache.len() >= MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES
        && let Some(evicted_path) = cache.keys().next().cloned()
    {
        cache.remove(&evicted_path);
    }

    cache.insert(path, value);
}

fn insert_workspace_cargo_toml(
    cache: &mut HashMap<PathBuf, CachedWorkspaceCargoToml>,
    path: PathBuf,
    value: CachedWorkspaceCargoToml,
) {
    if !cache.contains_key(&path)
        && cache.len() >= MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES
        && let Some(evicted_path) = cache.keys().next().cloned()
    {
        cache.remove(&evicted_path);
    }

    cache.insert(path, value);
}

async fn load_cargo_toml_document_tree(
    cargo_toml_path: PathBuf,
    toml_version: TomlVersion,
) -> Option<(PathBuf, tombi_document_tree::DocumentTree)> {
    let canonicalized_path = canonicalize_or_original(cargo_toml_path);
    let version = file_cache_version(&canonicalized_path);

    {
        let cache = DID_OPEN_CARGO_TOML_CACHE.read().await;
        if let Some(cached_cargo_toml) = cache.get(&canonicalized_path)
            && cached_cargo_toml.version == version
        {
            return Some((canonicalized_path, cached_cargo_toml.document_tree.clone()));
        }
    }

    let parsed_document_tree = tokio::task::spawn_blocking({
        let canonicalized_path = canonicalized_path.clone();
        move || {
            load_cargo_toml(&canonicalized_path, toml_version)
                .map(|(_, document_tree)| document_tree)
        }
    })
    .await
    .ok()
    .flatten()?;

    {
        let mut cache = DID_OPEN_CARGO_TOML_CACHE.write().await;
        insert_cargo_toml(
            &mut cache,
            canonicalized_path.clone(),
            CachedCargoToml {
                version,
                document_tree: parsed_document_tree.clone(),
            },
        );
    }

    Some((canonicalized_path, parsed_document_tree))
}

pub(crate) fn find_workspace_cargo_toml(
    cargo_toml_path: &Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
) -> Option<(PathBuf, tombi_ast::Root, tombi_document_tree::DocumentTree)> {
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

pub(crate) async fn load_workspace_cargo_toml(
    cargo_toml_path: &Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
) -> Option<(PathBuf, tombi_document_tree::DocumentTree)> {
    let cache_key = canonicalize_or_original(cargo_toml_path.to_path_buf());
    let cache_version = file_cache_version(&cache_key);

    {
        let cache = DID_OPEN_WORKSPACE_CARGO_TOML_CACHE.read().await;
        if let Some(cached_workspace_cargo_toml) = cache.get(&cache_key)
            && cached_workspace_cargo_toml.version == cache_version
        {
            if let Some(workspace_cargo_toml_path) =
                &cached_workspace_cargo_toml.workspace_cargo_toml_path
            {
                if let Some((workspace_cargo_toml_path, document_tree)) =
                    load_cargo_toml_document_tree(workspace_cargo_toml_path.clone(), toml_version)
                        .await
                {
                    if document_tree.contains_key("workspace") {
                        return Some((workspace_cargo_toml_path, document_tree));
                    }
                }
            } else {
                return None;
            }
        }
    }

    let workspace_cargo_toml = tokio::task::spawn_blocking({
        let cargo_toml_path = cache_key.clone();
        let workspace_path = workspace_path.map(str::to_owned);
        move || {
            find_workspace_cargo_toml(&cargo_toml_path, workspace_path.as_deref(), toml_version)
                .map(|(path, _root, document_tree)| (path, document_tree))
        }
    })
    .await
    .ok()
    .flatten();

    {
        let mut cache = DID_OPEN_WORKSPACE_CARGO_TOML_CACHE.write().await;
        insert_workspace_cargo_toml(
            &mut cache,
            cache_key,
            CachedWorkspaceCargoToml {
                version: cache_version,
                workspace_cargo_toml_path: workspace_cargo_toml
                    .as_ref()
                    .map(|(workspace_cargo_toml_path, _)| workspace_cargo_toml_path.clone()),
            },
        );
    }

    if let Some((workspace_cargo_toml_path, workspace_document_tree)) = workspace_cargo_toml {
        let mut cache = DID_OPEN_CARGO_TOML_CACHE.write().await;
        insert_cargo_toml(
            &mut cache,
            workspace_cargo_toml_path.clone(),
            CachedCargoToml {
                version: file_cache_version(&workspace_cargo_toml_path),
                document_tree: workspace_document_tree.clone(),
            },
        );

        return Some((workspace_cargo_toml_path, workspace_document_tree));
    }

    None
}

/// Get the workspace path from Cargo.toml
///
/// See: https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field
#[inline]
pub(crate) fn get_workspace_cargo_toml_path(
    document_tree: &tombi_document_tree::DocumentTree,
) -> Option<&str> {
    tombi_document_tree::dig_keys(document_tree, &["package", "workspace"]).and_then(
        |(_, workspace)| {
            if let tombi_document_tree::Value::String(workspace_path) = workspace {
                Some(workspace_path.value())
            } else {
                None
            }
        },
    )
}

/// Get the location of the workspace Cargo.toml.
pub(crate) fn goto_workspace(
    accessors: &[tombi_schema_store::Accessor],
    crate_cargo_toml_path: &std::path::Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
    jump_to_subcrate: bool,
) -> Result<Option<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    debug_assert!(matches!(
        accessors.last(),
        Some(tombi_schema_store::Accessor::Key(key)) if key == "workspace"
    ));

    let Some((workspace_cargo_toml_path, _, workspace_cargo_toml_document_tree)) =
        find_workspace_cargo_toml(crate_cargo_toml_path, workspace_path, toml_version)
    else {
        return Ok(None);
    };

    let keys = {
        let is_target_dependency = accessors.len() >= 3
            && (matches_accessors!(accessors[..3], ["target", _, "dependencies"])
                || matches_accessors!(accessors[..3], ["target", _, "dev-dependencies"])
                || matches_accessors!(accessors[..3], ["target", _, "build-dependencies"]));

        let start_index = if is_target_dependency { 2 } else { 0 };

        let mut sanitized_keys =
            if let Some(tombi_schema_store::Accessor::Key(key)) = accessors.get(start_index) {
                vec![sanitize_dependency_key(key)]
            } else {
                return Ok(None);
            };
        sanitized_keys.extend(accessors[start_index + 1..].iter().filter_map(|accessor| {
            if let tombi_schema_store::Accessor::Key(key) = accessor {
                Some(key.as_str())
            } else {
                None
            }
        }));
        sanitized_keys
    };

    let Some((key, value)) = tombi_document_tree::dig_keys(
        &workspace_cargo_toml_document_tree,
        &std::iter::once("workspace")
            .chain(keys[..keys.len() - 1].iter().copied())
            .collect_vec(),
    ) else {
        return Ok(None);
    };

    if jump_to_subcrate
        && matches!(
            keys.first(),
            Some(key) if *key == "dependencies" || *key == "dev-dependencies" || *key == "build-dependencies"
        )
        && let tombi_document_tree::Value::Table(table) = value
        && let Some(tombi_document_tree::Value::String(subcrate_path)) = table.get("path")
        && let Some((subcrate_cargo_toml_path, _, subcrate_document_tree)) = find_cargo_toml(
            &workspace_cargo_toml_path,
            std::path::Path::new(subcrate_path.value()),
            toml_version,
        )
        && let Some((_, tombi_document_tree::Value::String(package_name))) =
            tombi_document_tree::dig_keys(&subcrate_document_tree, &["package", "name"])
    {
        let Ok(subcrate_cargo_toml_uri) = tombi_uri::Uri::from_file_path(&subcrate_cargo_toml_path)
        else {
            return Ok(None);
        };

        return Ok(Some(tombi_extension::DefinitionLocation {
            uri: subcrate_cargo_toml_uri,
            range: package_name.unquoted_range(),
        }));
    }

    let Ok(workspace_cargo_toml_uri) = tombi_uri::Uri::from_file_path(&workspace_cargo_toml_path)
    else {
        return Ok(None);
    };

    Ok(Some(tombi_extension::DefinitionLocation {
        uri: workspace_cargo_toml_uri,
        range: key.unquoted_range(),
    }))
}

/// Get the location of the crate path in the workspace.
pub(crate) fn goto_dependency_crates(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_subcrate: bool,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    debug_assert!(
        matches_accessors!(accessors, ["workspace", "dependencies", _])
            || matches_accessors!(accessors, ["dependencies", _])
            || matches_accessors!(accessors, ["dev-dependencies", _])
            || matches_accessors!(accessors, ["build-dependencies", _])
            || matches_accessors!(accessors, ["target", _, "dependencies", _])
            || matches_accessors!(accessors, ["target", _, "dev-dependencies", _])
            || matches_accessors!(accessors, ["target", _, "build-dependencies", _])
    );

    let Some((tombi_schema_store::Accessor::Key(crate_name), crate_value)) =
        dig_accessors(workspace_document_tree, accessors)
    else {
        return Ok(Vec::with_capacity(0));
    };

    let is_workspace_cargo_toml =
        matches_accessors!(accessors[..accessors.len().min(1)], ["workspace"]);
    let mut locations = Vec::new();
    if let tombi_document_tree::Value::Table(table) = crate_value {
        if let Some(tombi_document_tree::Value::String(subcrate_path)) = table.get("path") {
            if let Some((subcrate_cargo_toml_path, _, subcrate_document_tree)) = find_cargo_toml(
                workspace_cargo_toml_path,
                std::path::Path::new(subcrate_path.value()),
                toml_version,
            ) && let Some((_, tombi_document_tree::Value::String(package_name))) =
                tombi_document_tree::dig_keys(&subcrate_document_tree, &["package", "name"])
            {
                if let Ok(subcrate_cargo_toml_uri) =
                    tombi_uri::Uri::from_file_path(&subcrate_cargo_toml_path)
                {
                    locations.push(tombi_extension::DefinitionLocation {
                        uri: subcrate_cargo_toml_uri,
                        range: package_name.unquoted_range(),
                    });
                }
            }
        } else if let Some(tombi_document_tree::Value::Boolean(has_workspace)) =
            table.get("workspace")
            && has_workspace.value()
        {
            let mut accessors = accessors.iter().cloned().collect_vec();
            accessors.push(tombi_schema_store::Accessor::Key("workspace".to_string()));
            if is_workspace_cargo_toml {
                locations.extend(goto_definition_for_workspace_cargo_toml(
                    workspace_document_tree,
                    &accessors,
                    workspace_cargo_toml_path,
                    toml_version,
                    jump_to_subcrate,
                )?);
            } else {
                locations.extend(goto_definition_for_crate_cargo_toml(
                    workspace_document_tree,
                    &accessors,
                    workspace_cargo_toml_path,
                    toml_version,
                    jump_to_subcrate,
                )?);
            }
        }
    }
    if is_workspace_cargo_toml {
        let platforms = if let Some((_, tombi_document_tree::Value::Table(targets))) =
            tombi_document_tree::dig_keys(workspace_document_tree, &["target"])
        {
            targets
                .values()
                .filter_map(|value| {
                    if let tombi_document_tree::Value::Table(platform) = value {
                        Some(platform)
                    } else {
                        None
                    }
                })
                .collect_vec()
        } else {
            Vec::with_capacity(0)
        };
        for crate_location in goto_workspace_member_crates(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
            "members",
        )? {
            let Some((_, crate_document_tree)) =
                load_cargo_toml(&crate_location.cargo_toml_path, toml_version)
            else {
                continue;
            };

            for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some((crate_key, _)) = tombi_document_tree::dig_keys(
                    &crate_document_tree,
                    &[dependency_key, crate_name],
                ) && let Some(mut definition_location) =
                    Option::<tombi_extension::DefinitionLocation>::from(crate_location.clone())
                {
                    definition_location.range = crate_key.unquoted_range();
                    locations.push(definition_location);
                }
            }
            for platform in &platforms {
                for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    if let Some((crate_key, _)) =
                        tombi_document_tree::dig_keys(platform, &[dependency_key, crate_name])
                        && let Some(mut definition_location) =
                            Option::<tombi_extension::DefinitionLocation>::from(
                                crate_location.clone(),
                            )
                    {
                        definition_location.range = crate_key.unquoted_range();
                        locations.push(definition_location);
                    }
                }
            }
        }
    }

    Ok(locations)
}

pub(crate) fn goto_crate_package(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Option<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    debug_assert!(
        matches_accessors!(accessors, ["workspace", "dependencies", _, "path"])
            || matches_accessors!(accessors, ["dependencies", _, "path"])
            || matches_accessors!(accessors, ["dev-dependencies", _, "path"])
            || matches_accessors!(accessors, ["build-dependencies", _, "path"])
            || matches_accessors!(accessors, ["target", _, "dependencies", _, "path"])
            || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, "path"])
            || matches_accessors!(accessors, ["target", _, "build-dependencies", _, "path"])
    );

    let Some((_, value)) = dig_accessors(workspace_document_tree, accessors) else {
        return Ok(None);
    };

    if value.value_type() == tombi_document_tree::ValueType::String {
        let subcrate_path = match value {
            tombi_document_tree::Value::String(path) => path,
            _ => unreachable!(),
        };

        if let Some((subcrate_cargo_toml_path, _, subcrate_document_tree)) = find_cargo_toml(
            workspace_cargo_toml_path,
            std::path::Path::new(subcrate_path.value()),
            toml_version,
        ) && let Some((_, tombi_document_tree::Value::String(package_name))) =
            tombi_document_tree::dig_keys(&subcrate_document_tree, &["package", "name"])
        {
            let Ok(subcrate_cargo_toml_uri) =
                tombi_uri::Uri::from_file_path(&subcrate_cargo_toml_path)
            else {
                return Ok(None);
            };

            return Ok(Some(tombi_extension::DefinitionLocation {
                uri: subcrate_cargo_toml_uri,
                range: package_name.unquoted_range(),
            }));
        }
    }

    Ok(None)
}

pub(crate) fn goto_bin_path_target(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    cargo_toml_path: &std::path::Path,
) -> Result<Option<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    debug_assert!(matches_accessors!(accessors, ["bin", _, "path"]));

    let Some((_, tombi_document_tree::Value::String(path_value))) =
        dig_accessors(document_tree, accessors)
    else {
        return Ok(None);
    };

    let Some(uri) =
        get_uri_relative_to_cargo_toml(std::path::Path::new(path_value.value()), cargo_toml_path)
    else {
        return Ok(None);
    };

    Ok(Some(tombi_extension::DefinitionLocation {
        uri,
        range: tombi_text::Range::default(),
    }))
}

#[inline]
pub(crate) fn sanitize_dependency_key(key: &str) -> &str {
    if matches!(key, "dev-dependencies" | "build-dependencies") {
        "dependencies"
    } else {
        key
    }
}

pub(crate) fn extract_member_patterns<'a>(
    workspace_document_tree: &'a tombi_document_tree::DocumentTree,
    accessors: &'a [tombi_schema_store::Accessor],
    members_key: &'static str,
) -> Vec<&'a tombi_document_tree::String> {
    if matches_accessors!(accessors, ["workspace", members_key, _]) {
        let Some((_, tombi_document_tree::Value::String(member))) =
            dig_accessors(workspace_document_tree, accessors)
        else {
            return vec![];
        };
        vec![member]
    } else {
        match tombi_document_tree::dig_keys(workspace_document_tree, &["workspace", members_key]) {
            Some((_, tombi_document_tree::Value::Array(members))) => members
                .iter()
                .filter_map(|member| match member {
                    tombi_document_tree::Value::String(member_pattern) => Some(member_pattern),
                    _ => None,
                })
                .collect_vec(),
            _ => vec![],
        }
    }
}

pub(crate) fn extract_exclude_patterns(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
) -> Vec<&tombi_document_tree::String> {
    match tombi_document_tree::dig_keys(workspace_document_tree, &["workspace", "exclude"]) {
        Some((_, tombi_document_tree::Value::Array(exclude))) => exclude
            .iter()
            .filter_map(|member| match member {
                tombi_document_tree::Value::String(member_pattern) => Some(member_pattern),
                _ => None,
            })
            .collect_vec(),
        _ => Vec::with_capacity(0),
    }
}

pub(crate) fn find_package_cargo_toml_paths<'a>(
    member_patterns: &'a [&'a tombi_document_tree::String],
    exclude_patterns: &'a [&'a tombi_document_tree::String],
    workspace_dir_path: &'a std::path::Path,
) -> impl Iterator<Item = (&'a tombi_document_tree::String, std::path::PathBuf)> + 'a {
    let exclude_patterns = exclude_patterns
        .iter()
        .filter_map(|pattern| glob::Pattern::new(pattern.value()).ok())
        .collect_vec();

    member_patterns
        .iter()
        .filter_map(move |&member_pattern| {
            let mut cargo_toml_paths = vec![];

            let mut member_pattern_path =
                std::path::Path::new(member_pattern.value()).to_path_buf();
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

                let cargo_toml_path = candidate_path.join("Cargo.toml");
                if !cargo_toml_path.is_file() {
                    continue;
                }

                let is_excluded = exclude_patterns.iter().any(|exclude_pattern| {
                    exclude_pattern.matches(&cargo_toml_path.to_string_lossy())
                });

                if !is_excluded {
                    cargo_toml_paths.push((member_pattern, cargo_toml_path));
                }
            }

            (!cargo_toml_paths.is_empty()).then_some(cargo_toml_paths)
        })
        .flatten()
}

pub(crate) fn goto_definition_for_workspace_cargo_toml(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_subcrate: bool,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    if matches_accessors!(accessors, ["workspace", "dependencies", _]) {
        goto_dependency_crates(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
            jump_to_subcrate,
        )
    } else if matches_accessors!(accessors, ["workspace", "dependencies", _, "path"]) {
        goto_crate_package(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
        )
        .map(|location| location.into_iter().collect())
    } else if matches_accessors!(accessors, ["workspace", "members"])
        || matches_accessors!(accessors, ["workspace", "members", _])
    {
        goto_workspace_member_crates(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
            "members",
        )
        .map(|locations| locations.into_iter().filter_map(Into::into).collect_vec())
    } else if matches_accessors!(accessors, ["workspace", "default-members"])
        || matches_accessors!(accessors, ["workspace", "default-members", _])
    {
        goto_workspace_member_crates(
            workspace_document_tree,
            accessors,
            workspace_cargo_toml_path,
            toml_version,
            "default-members",
        )
        .map(|locations| locations.into_iter().filter_map(Into::into).collect_vec())
    } else {
        Ok(Vec::with_capacity(0))
    }
}

pub(crate) fn goto_definition_for_crate_cargo_toml(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    jump_to_subcrate: bool,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    let location = if matches_accessors!(accessors, ["dependencies", _])
        || matches_accessors!(accessors, ["dev-dependencies", _])
        || matches_accessors!(accessors, ["build-dependencies", _])
        || matches_accessors!(accessors, ["target", _, "dependencies", _])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _])
        || matches_accessors!(accessors, ["target", _, "build-dependencies", _])
    {
        return goto_dependency_crates(
            document_tree,
            accessors,
            cargo_toml_path,
            toml_version,
            jump_to_subcrate,
        );
    } else if accessors.last() == Some(&tombi_schema_store::Accessor::Key("workspace".to_string()))
    {
        goto_workspace(
            accessors,
            cargo_toml_path,
            get_workspace_cargo_toml_path(document_tree),
            toml_version,
            jump_to_subcrate,
        )
    } else if matches_accessors!(accessors, ["dependencies", _, "path"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "path"])
        || matches_accessors!(accessors, ["build-dependencies", _, "path"])
        || matches_accessors!(accessors, ["target", _, "dependencies", _, "path"])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, "path"])
        || matches_accessors!(accessors, ["target", _, "build-dependencies", _, "path"])
    {
        goto_crate_package(document_tree, accessors, cargo_toml_path, toml_version)
    } else if matches_accessors!(accessors, ["bin", _, "path"]) {
        goto_bin_path_target(document_tree, accessors, cargo_toml_path)
    } else {
        Ok(None)
    }?;

    match location {
        Some(location) => Ok(vec![location]),
        None => Ok(Vec::with_capacity(0)),
    }
}

pub(crate) fn goto_workspace_member_crates(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    members_key: &'static str,
) -> Result<Vec<CrateLocation>, tower_lsp::jsonrpc::Error> {
    let member_patterns = extract_member_patterns(workspace_document_tree, accessors, members_key);
    if member_patterns.is_empty() {
        return Ok(Vec::with_capacity(0));
    }

    let Some(workspace_dir_path) = workspace_cargo_toml_path.parent() else {
        return Ok(Vec::with_capacity(0));
    };

    let exclude_patterns = extract_exclude_patterns(workspace_document_tree);

    let mut locations = Vec::new();
    for (_, cargo_toml_path) in
        find_package_cargo_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some((_, member_document_tree)) = load_cargo_toml(&cargo_toml_path, toml_version)
        else {
            continue;
        };

        let Some((_, tombi_document_tree::Value::String(package_name))) =
            tombi_document_tree::dig_keys(&member_document_tree, &["package", "name"])
        else {
            continue;
        };

        locations.push(CrateLocation {
            cargo_toml_path,
            package_name_key_range: package_name.unquoted_range(),
        });
    }

    Ok(locations)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{Mutex, OnceLock},
        time::Duration,
    };

    use tombi_ast::AstNode;
    use tombi_document_tree::TryIntoDocumentTree;

    use super::*;

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn test_document_tree() -> tombi_document_tree::DocumentTree {
        let root = tombi_ast::Root::cast(
            tombi_parser::parse(
                r#"
                [package]
                name = "example"
                version = "0.1.0"
                "#,
            )
            .into_syntax_node(),
        )
        .expect("expected root");
        root.try_into_document_tree(TomlVersion::default())
            .expect("expected document tree")
    }

    async fn clear_caches() {
        DID_OPEN_CARGO_TOML_CACHE.write().await.clear();
        DID_OPEN_WORKSPACE_CARGO_TOML_CACHE.write().await.clear();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reload_workspace_lookup_when_cached_workspace_loses_workspace_table() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_caches().await;

        let temp_dir = tempfile::tempdir().expect("expected temp dir");
        let workspace_cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let member_dir = temp_dir.path().join("member");
        fs::create_dir(&member_dir).expect("expected member dir");
        let member_cargo_toml_path = member_dir.join("Cargo.toml");

        fs::write(
            &workspace_cargo_toml_path,
            r#"
            [workspace]
            members = ["member"]
            "#,
        )
        .expect("expected workspace Cargo.toml");
        fs::write(
            &member_cargo_toml_path,
            r#"
            [package]
            name = "member"
            version = "0.1.0"
            workspace = ".."
            "#,
        )
        .expect("expected member Cargo.toml");

        let first =
            load_workspace_cargo_toml(&member_cargo_toml_path, Some(".."), TomlVersion::default())
                .await;
        assert!(first.is_some());

        std::thread::sleep(Duration::from_millis(5));
        fs::write(
            &workspace_cargo_toml_path,
            r#"
            [package]
            name = "workspace-root"
            version = "0.1.0"
            "#,
        )
        .expect("expected rewritten Cargo.toml");

        let second =
            load_workspace_cargo_toml(&member_cargo_toml_path, Some(".."), TomlVersion::default())
                .await;
        assert!(second.is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn keeps_did_open_caches_bounded() {
        let _guard = test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_caches().await;

        {
            let mut cargo_toml_cache = DID_OPEN_CARGO_TOML_CACHE.write().await;
            for index in 0..=MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES {
                insert_cargo_toml(
                    &mut cargo_toml_cache,
                    PathBuf::from(format!("/tmp/cargo-{index}/Cargo.toml")),
                    CachedCargoToml {
                        version: Some(index as u64),
                        document_tree: test_document_tree(),
                    },
                );
            }
            assert_eq!(
                cargo_toml_cache.len(),
                MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES
            );
        }

        {
            let mut workspace_cargo_toml_cache = DID_OPEN_WORKSPACE_CARGO_TOML_CACHE.write().await;
            for index in 0..=MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES {
                insert_workspace_cargo_toml(
                    &mut workspace_cargo_toml_cache,
                    PathBuf::from(format!("/tmp/member-{index}/Cargo.toml")),
                    CachedWorkspaceCargoToml {
                        version: Some(index as u64),
                        workspace_cargo_toml_path: Some(PathBuf::from(format!(
                            "/tmp/workspace-{index}/Cargo.toml"
                        ))),
                    },
                );
            }
            assert_eq!(
                workspace_cargo_toml_cache.len(),
                MAX_DID_OPEN_CARGO_TOML_CACHE_ENTRIES
            );
        }
    }
}
