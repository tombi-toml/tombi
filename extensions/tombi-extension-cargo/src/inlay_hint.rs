use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
};

use futures::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{TryIntoDocumentTree, Value, dig_keys};
use tombi_extension::{InlayHint, fetch_cached_remote_json, file_cache_version, get_or_load_json};
use tombi_hashmap::{HashMap, HashSet};

use crate::{
    cargo_lock::{
        CargoLock, CargoLockPackage, find_cargo_lock_path, load_cached_cargo_lock,
        load_cargo_lock_from_path,
    },
    dependency_package_name, find_workspace_cargo_toml, get_workspace_path, load_cargo_toml,
    workspace::{extract_exclude_patterns, find_package_cargo_toml_paths},
};

const RESOLVED_VERSION_TOOLTIP: &str = "Resolved version in Cargo.lock";
const LOCAL_PATH_VERSION_TOOLTIP: &str = "Version from local dependency Cargo.toml";
const WORKSPACE_INHERITED_VALUE_TOOLTIP: &str = "Inherited value from workspace";
const MAX_WORKSPACE_VALUE_HINT_CHARS: usize = 80;
const CARGO_EXTENSION_ID: &str = "tombi-toml/cargo";
const INLAY_HINT_LOCKFILE_KEY: &str = "inlay_hint.lockfile";
const LOCAL_MANIFEST_PREFETCH_CONCURRENCY: usize = 8;
const WORKSPACE_PACKAGE_ITEMS: [&str; 16] = [
    "authors",
    "categories",
    "description",
    "documentation",
    "edition",
    "exclude",
    "homepage",
    "include",
    "keywords",
    "license-file",
    "license",
    "publish",
    "readme",
    "repository",
    "rust-version",
    "version",
];

enum CargoInlayHintFeature {
    DependencyVersion,
    DefaultFeatures,
    WorkspaceValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
struct CrateName(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
struct CrateVersion(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
struct DependencyCrateName(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResolvedDependencyVersion {
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrateResolvedDependencies {
    by_dependency: tombi_hashmap::HashMap<DependencyCrateName, ResolvedDependencyVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CargoLockInlayCacheData {
    crates: tombi_hashmap::HashMap<
        CrateName,
        tombi_hashmap::HashMap<CrateVersion, CrateResolvedDependencies>,
    >,
}

struct DependencyVersionHint {
    dependency_name: String,
    position: tombi_text::Position,
    current_version: Option<String>,
    always_show: bool,
    local_version_source: Option<LocalVersionSource>,
}

enum LocalVersionSource {
    Path(String),
    WorkspaceDependency(String),
}

struct DefaultFeaturesHint {
    position: tombi_text::Position,
    label: String,
    tooltip: String,
}

struct CurrentPackage<'a> {
    name: &'a str,
    version: String,
}

#[derive(Clone)]
struct WorkspaceMemberPackage {
    name: String,
    version: String,
}

#[derive(Default)]
struct LocalManifestCache {
    manifests: HashMap<PathBuf, tombi_document_tree::DocumentTree>,
    workspace_manifests: HashMap<PathBuf, Option<PathBuf>>,
    workspace_member_packages: HashMap<PathBuf, Vec<WorkspaceMemberPackage>>,
}

struct LocalManifestData {
    cargo_toml_path: PathBuf,
    document_tree: tombi_document_tree::DocumentTree,
}

#[derive(Default)]
struct LocalManifestRequests {
    path_dependencies: HashSet<String>,
    workspace_dependencies: HashSet<String>,
}

enum WorkspaceDocumentTree<'a> {
    Current(&'a tombi_document_tree::DocumentTree),
    External(tombi_document_tree::DocumentTree),
}

impl WorkspaceDocumentTree<'_> {
    fn as_tree(&self) -> &tombi_document_tree::DocumentTree {
        match self {
            Self::Current(document_tree) => document_tree,
            Self::External(document_tree) => document_tree,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CratesIoVersionDetailResponse {
    version: CratesIoVersion,
}

#[derive(Debug, Deserialize)]
struct CratesIoVersion {
    features: HashMap<String, Vec<String>>,
}

pub async fn inlay_hint(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<InlayHint>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }

    if !cargo_inlay_hint_root_enabled(features) {
        return Ok(None);
    }

    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    let dependency_version_enabled =
        cargo_inlay_hint_enabled(features, CargoInlayHintFeature::DependencyVersion);
    let default_features_enabled =
        cargo_inlay_hint_enabled(features, CargoInlayHintFeature::DefaultFeatures);
    let (cargo_lock_cache, local_manifest_cache) = tokio::join!(
        async {
            if dependency_version_enabled {
                load_cargo_lock_cache(&cargo_toml_path, toml_version).await
            } else {
                None
            }
        },
        async {
            if dependency_version_enabled || default_features_enabled {
                preload_local_manifest_cache(
                    document_tree,
                    &cargo_toml_path,
                    visible_range,
                    toml_version,
                    dependency_version_enabled,
                    default_features_enabled,
                    has_visible_workspace_value_targets(document_tree, visible_range),
                )
                .await
            } else {
                LocalManifestCache::default()
            }
        }
    );

    let sync_text_document_uri = text_document_uri.clone();
    let sync_document_tree = document_tree.clone();
    let sync_features = features.cloned();
    let sync_hints = tokio::task::spawn_blocking(move || {
        inlay_hint_impl(
            &sync_text_document_uri,
            &sync_document_tree,
            visible_range,
            cargo_lock_cache,
            local_manifest_cache,
            toml_version,
            sync_features.as_ref(),
        )
    })
    .await
    .map_err(|_| tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::InternalError))??;

    let mut hints = sync_hints.unwrap_or_default();

    if cargo_inlay_hint_root_enabled(features)
        && cargo_inlay_hint_enabled(features, CargoInlayHintFeature::DefaultFeatures)
    {
        hints.extend(
            registry_default_features_inlay_hints(
                text_document_uri,
                document_tree,
                visible_range,
                toml_version,
                offline,
                cache_options,
            )
            .await?,
        );
    }

    if hints.is_empty() {
        Ok(None)
    } else {
        hints.sort_by_key(|hint| hint.position);
        Ok(Some(hints))
    }
}

fn inlay_hint_impl(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
    cargo_lock_cache: Option<CargoLockInlayCacheData>,
    mut local_manifest_cache: LocalManifestCache,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<InlayHint>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }

    if !cargo_inlay_hint_root_enabled(features) {
        return Ok(None);
    }

    let dependency_version_enabled =
        cargo_inlay_hint_enabled(features, CargoInlayHintFeature::DependencyVersion);
    let default_features_enabled =
        cargo_inlay_hint_enabled(features, CargoInlayHintFeature::DefaultFeatures);
    let workspace_value_enabled =
        cargo_inlay_hint_enabled(features, CargoInlayHintFeature::WorkspaceValue);
    if !dependency_version_enabled && !default_features_enabled && !workspace_value_enabled {
        return Ok(None);
    }

    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    let mut hints = Vec::new();

    if workspace_value_enabled {
        collect_workspace_value_inlay_hints(
            document_tree,
            &cargo_toml_path,
            &mut local_manifest_cache,
            toml_version,
            visible_range,
            &mut hints,
        );
    }

    if dependency_version_enabled || default_features_enabled {
        for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
            collect_dependency_inlay_hints(
                document_tree,
                &[dependency_key],
                &cargo_toml_path,
                cargo_lock_cache.as_ref(),
                &mut local_manifest_cache,
                toml_version,
                visible_range,
                dependency_version_enabled,
                default_features_enabled,
                &mut hints,
            );
        }

        collect_dependency_inlay_hints(
            document_tree,
            &["workspace", "dependencies"],
            &cargo_toml_path,
            cargo_lock_cache.as_ref(),
            &mut local_manifest_cache,
            toml_version,
            visible_range,
            dependency_version_enabled,
            default_features_enabled,
            &mut hints,
        );

        if let Some((_, Value::Table(targets))) = dig_keys(document_tree, &["target"]) {
            for (target_key, target_value) in targets.key_values() {
                let Value::Table(_) = target_value else {
                    continue;
                };

                for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    collect_dependency_inlay_hints(
                        document_tree,
                        &["target", target_key.value.as_str(), dependency_key],
                        &cargo_toml_path,
                        cargo_lock_cache.as_ref(),
                        &mut local_manifest_cache,
                        toml_version,
                        visible_range,
                        dependency_version_enabled,
                        default_features_enabled,
                        &mut hints,
                    );
                }
            }
        }
    }

    if hints.is_empty() {
        Ok(None)
    } else {
        hints.sort_by_key(|hint| hint.position);
        Ok(Some(hints))
    }
}

fn collect_workspace_value_inlay_hints(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
    visible_range: tombi_text::Range,
    hints: &mut Vec<InlayHint>,
) {
    if !has_visible_workspace_value_targets(document_tree, visible_range) {
        return;
    }

    let Some(workspace_document_tree) = workspace_document_tree(
        document_tree,
        cargo_toml_path,
        local_manifest_cache,
        toml_version,
    ) else {
        return;
    };
    let workspace_document_tree = workspace_document_tree.as_tree();

    collect_workspace_package_inlay_hints(
        document_tree,
        workspace_document_tree,
        visible_range,
        hints,
    );
    collect_workspace_lints_inlay_hints(
        document_tree,
        workspace_document_tree,
        visible_range,
        hints,
    );
}

fn collect_workspace_package_inlay_hints(
    document_tree: &tombi_document_tree::DocumentTree,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
    hints: &mut Vec<InlayHint>,
) {
    for package_item in WORKSPACE_PACKAGE_ITEMS {
        let Some((_, Value::Boolean(workspace))) =
            dig_keys(document_tree, &["package", package_item, "workspace"])
        else {
            continue;
        };
        if !workspace.value() {
            continue;
        }

        let Some((_, workspace_value)) = dig_keys(
            workspace_document_tree,
            &["workspace", "package", package_item],
        ) else {
            continue;
        };

        push_workspace_value_hint(workspace, workspace_value, visible_range, hints);
    }
}

fn collect_workspace_lints_inlay_hints(
    document_tree: &tombi_document_tree::DocumentTree,
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
    hints: &mut Vec<InlayHint>,
) {
    let Some((_, Value::Boolean(workspace))) = dig_keys(document_tree, &["lints", "workspace"])
    else {
        return;
    };
    if !workspace.value() {
        return;
    }

    let Some((_, workspace_value)) = dig_keys(workspace_document_tree, &["workspace", "lints"])
    else {
        return;
    };

    push_workspace_value_hint(workspace, workspace_value, visible_range, hints);
}

fn has_visible_workspace_value_targets(
    document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
) -> bool {
    WORKSPACE_PACKAGE_ITEMS.iter().any(|package_item| {
        matches!(
            dig_keys(document_tree, &["package", package_item, "workspace"]),
            Some((_, Value::Boolean(workspace)))
                if workspace.value()
                    && tombi_text::Range::at(workspace.range().end).intersects(visible_range)
        )
    }) || matches!(
        dig_keys(document_tree, &["lints", "workspace"]),
        Some((_, Value::Boolean(workspace)))
            if workspace.value()
                && tombi_text::Range::at(workspace.range().end).intersects(visible_range)
    )
}

fn push_workspace_value_hint(
    workspace: &tombi_document_tree::Boolean,
    workspace_value: &Value,
    visible_range: tombi_text::Range,
    hints: &mut Vec<InlayHint>,
) {
    if !tombi_text::Range::at(workspace.range().end).intersects(visible_range) {
        return;
    }

    let Some(label) = workspace_value_hint_label(workspace_value) else {
        return;
    };

    hints.push(InlayHint {
        position: workspace.range().end,
        label,
        kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
        tooltip: Some(WORKSPACE_INHERITED_VALUE_TOOLTIP.to_string()),
        padding_left: Some(true),
        padding_right: Some(false),
    });
}

fn collect_dependency_inlay_hints(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_keys: &[&str],
    cargo_toml_path: &Path,
    cargo_lock_cache: Option<&CargoLockInlayCacheData>,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
    visible_range: tombi_text::Range,
    dependency_version_enabled: bool,
    default_features_enabled: bool,
    hints: &mut Vec<InlayHint>,
) {
    let Some((_, Value::Table(dependencies))) = dig_keys(document_tree, dependency_keys) else {
        return;
    };

    let mut version_hints = Vec::new();
    let mut default_feature_hints = Vec::new();

    for (dependency_key, dependency_value) in dependencies.key_values() {
        if dependency_version_enabled
            && let Some(version_hint) =
                dependency_version_hint(&dependency_key.value, dependency_value)
            && tombi_text::Range::at(version_hint.position).intersects(visible_range)
        {
            version_hints.push(version_hint);
        }

        if default_features_enabled
            && let Some(default_features_hint) = dependency_default_features_hint(
                document_tree,
                &dependency_key.value,
                dependency_value,
                cargo_toml_path,
                local_manifest_cache,
                toml_version,
            )
            && tombi_text::Range::at(default_features_hint.position).intersects(visible_range)
        {
            default_feature_hints.push(default_features_hint);
        }
    }

    if version_hints.is_empty() && default_feature_hints.is_empty() {
        return;
    }

    let current_package = if dependency_version_enabled && !version_hints.is_empty() {
        if dependency_keys == ["workspace", "dependencies"] {
            None
        } else {
            current_package(
                document_tree,
                cargo_toml_path,
                local_manifest_cache,
                toml_version,
            )
        }
    } else {
        None
    };

    let workspace_member_packages = if dependency_version_enabled && !version_hints.is_empty() {
        if dependency_keys == ["workspace", "dependencies"] {
            workspace_member_packages(
                document_tree,
                cargo_toml_path,
                local_manifest_cache,
                toml_version,
            )
        } else {
            None
        }
    } else {
        None
    };

    let workspace_resolved_versions = if dependency_version_enabled
        && !version_hints.is_empty()
        && dependency_keys == ["workspace", "dependencies"]
    {
        cargo_lock_cache.map_or_else(HashMap::new, |cargo_lock_cache| {
            workspace_dependency_lock_versions(
                cargo_lock_cache,
                workspace_member_packages.as_deref(),
                version_hints
                    .iter()
                    .map(|version_hint| version_hint.dependency_name.as_str()),
            )
        })
    } else {
        HashMap::new()
    };

    for version_hint in version_hints {
        let DependencyVersionHint {
            dependency_name,
            position,
            current_version,
            always_show,
            local_version_source,
        } = version_hint;

        let resolved_version = if dependency_keys == ["workspace", "dependencies"] {
            workspace_resolved_versions
                .get(&dependency_name)
                .cloned()
                .map(|resolved_version| (resolved_version, RESOLVED_VERSION_TOOLTIP))
        } else {
            cargo_lock_cache
                .and_then(|cargo_lock_cache| {
                    cargo_lock_dependency_version(
                        cargo_lock_cache,
                        dependency_keys,
                        &dependency_name,
                        current_package.as_ref(),
                        workspace_member_packages.as_deref(),
                    )
                })
                .map(|resolved_version| (resolved_version, RESOLVED_VERSION_TOOLTIP))
        }
        .or_else(|| {
            dependency_local_version(
                document_tree,
                local_version_source.as_ref(),
                cargo_toml_path,
                local_manifest_cache,
                toml_version,
            )
            .map(|resolved_version| (resolved_version, LOCAL_PATH_VERSION_TOOLTIP))
        });

        let Some((resolved_version, tooltip)) = resolved_version else {
            continue;
        };

        let Some(label) =
            version_hint_label(current_version.as_deref(), &resolved_version, always_show)
        else {
            continue;
        };

        hints.push(InlayHint {
            position,
            label,
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(tooltip.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        });
    }

    hints.extend(default_feature_hints.into_iter().map(|hint| InlayHint {
        position: hint.position,
        label: hint.label,
        kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
        tooltip: Some(hint.tooltip),
        padding_left: Some(true),
        padding_right: Some(false),
    }));
}
fn dependency_version_hint(
    dependency_key: &str,
    dependency_value: &Value,
) -> Option<DependencyVersionHint> {
    let dependency_name = dependency_package_name(dependency_key, dependency_value).to_string();

    match dependency_value {
        Value::String(version) => Some(DependencyVersionHint {
            dependency_name,
            position: version.range().end,
            current_version: Some(version.value().to_string()),
            always_show: false,
            local_version_source: None,
        }),
        Value::Table(table) => {
            if let Some(Value::String(version)) = table.get("version") {
                return Some(DependencyVersionHint {
                    dependency_name,
                    position: version.range().end,
                    current_version: Some(version.value().to_string()),
                    always_show: false,
                    local_version_source: None,
                });
            }

            if let Some(Value::Boolean(workspace)) = table.get("workspace")
                && workspace.value()
            {
                return Some(DependencyVersionHint {
                    dependency_name,
                    position: workspace.range().end,
                    current_version: None,
                    always_show: true,
                    local_version_source: Some(LocalVersionSource::WorkspaceDependency(
                        dependency_key.to_string(),
                    )),
                });
            }

            if let Some(Value::String(path)) = table.get("path") {
                return Some(DependencyVersionHint {
                    dependency_name,
                    position: path.range().end,
                    current_version: None,
                    always_show: false,
                    local_version_source: Some(LocalVersionSource::Path(path.value().to_string())),
                });
            }

            if let Some(Value::String(git)) = table.get("git") {
                return Some(DependencyVersionHint {
                    dependency_name,
                    position: git.range().end,
                    current_version: None,
                    always_show: false,
                    local_version_source: None,
                });
            }

            None
        }
        _ => None,
    }
}

fn dependency_local_version(
    document_tree: &tombi_document_tree::DocumentTree,
    source: Option<&LocalVersionSource>,
    cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
) -> Option<String> {
    match source? {
        LocalVersionSource::Path(path) => {
            let (dependency_cargo_toml_path, dependency_document_tree) =
                load_local_dependency_document_tree_cached(
                    local_manifest_cache,
                    cargo_toml_path,
                    path,
                    toml_version,
                )?;
            package_version(
                &dependency_document_tree,
                &dependency_cargo_toml_path,
                local_manifest_cache,
                toml_version,
            )
        }
        LocalVersionSource::WorkspaceDependency(dependency_key) => {
            workspace_path_dependency_version(
                document_tree,
                dependency_key,
                cargo_toml_path,
                local_manifest_cache,
                toml_version,
            )
        }
    }
}

fn workspace_path_dependency_version(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_key: &str,
    cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
) -> Option<String> {
    if document_tree.contains_key("workspace") {
        let (_, workspace_dependency_value) = dig_keys(
            document_tree,
            &["workspace", "dependencies", dependency_key],
        )?;
        let Value::Table(workspace_dependency_table) = workspace_dependency_value else {
            return None;
        };
        let Some(Value::String(path)) = workspace_dependency_table.get("path") else {
            return None;
        };
        let (dependency_cargo_toml_path, dependency_document_tree) =
            load_local_dependency_document_tree_cached(
                local_manifest_cache,
                cargo_toml_path,
                path.value(),
                toml_version,
            )?;
        return package_version(
            &dependency_document_tree,
            &dependency_cargo_toml_path,
            local_manifest_cache,
            toml_version,
        );
    }

    let (workspace_cargo_toml_path, workspace_document_tree) = find_workspace_cargo_toml_cached(
        local_manifest_cache,
        cargo_toml_path,
        get_workspace_path(document_tree),
        toml_version,
    )?;
    let (_, workspace_dependency_value) = dig_keys(
        &workspace_document_tree,
        &["workspace", "dependencies", dependency_key],
    )?;
    let Value::Table(workspace_dependency_table) = workspace_dependency_value else {
        return None;
    };
    let Some(Value::String(path)) = workspace_dependency_table.get("path") else {
        return None;
    };
    let (dependency_cargo_toml_path, dependency_document_tree) =
        load_local_dependency_document_tree_cached(
            local_manifest_cache,
            &workspace_cargo_toml_path,
            path.value(),
            toml_version,
        )?;

    package_version(
        &dependency_document_tree,
        &dependency_cargo_toml_path,
        local_manifest_cache,
        toml_version,
    )
}

fn dependency_default_features_hint(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_key: &str,
    dependency_value: &Value,
    cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
) -> Option<DefaultFeaturesHint> {
    let Value::Table(table) = dependency_value else {
        return None;
    };

    let Some(Value::Array(features)) = table.get("features") else {
        return None;
    };

    if dependency_table_default_features_disabled(table) {
        return None;
    }

    let default_features = dependency_default_features(
        document_tree,
        dependency_key,
        dependency_value,
        cargo_toml_path,
        local_manifest_cache,
        toml_version,
    )?;

    build_default_features_hint(
        features.range().end,
        default_features,
        &collect_feature_names(features),
    )
}

fn dependency_default_features(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_key: &str,
    dependency_value: &Value,
    cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
) -> Option<Vec<String>> {
    let Value::Table(table) = dependency_value else {
        return None;
    };

    if let Some(Value::String(path)) = table.get("path") {
        let (_, dependency_document_tree) = load_local_dependency_document_tree_cached(
            local_manifest_cache,
            cargo_toml_path,
            path.value(),
            toml_version,
        )?;
        return package_default_features(&dependency_document_tree);
    }

    let Some(Value::Boolean(workspace)) = table.get("workspace") else {
        return None;
    };
    if !workspace.value() {
        return None;
    }

    let (workspace_cargo_toml_path, workspace_document_tree) = find_workspace_cargo_toml_cached(
        local_manifest_cache,
        cargo_toml_path,
        get_workspace_path(document_tree),
        toml_version,
    )?;
    let (_, workspace_dependency_value) = dig_keys(
        &workspace_document_tree,
        &["workspace", "dependencies", dependency_key],
    )?;
    let Value::Table(workspace_dependency_table) = workspace_dependency_value else {
        return None;
    };

    if dependency_table_default_features_disabled(workspace_dependency_table) {
        return None;
    }

    let Some(Value::String(path)) = workspace_dependency_table.get("path") else {
        return None;
    };

    let (_, dependency_document_tree) = load_local_dependency_document_tree_cached(
        local_manifest_cache,
        &workspace_cargo_toml_path,
        path.value(),
        toml_version,
    )?;
    package_default_features(&dependency_document_tree)
}

async fn preload_local_manifest_cache(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    visible_range: tombi_text::Range,
    toml_version: TomlVersion,
    dependency_version_enabled: bool,
    default_features_enabled: bool,
    preload_workspace_manifest: bool,
) -> LocalManifestCache {
    let requests = collect_local_manifest_requests(
        document_tree,
        visible_range,
        dependency_version_enabled,
        default_features_enabled,
    );
    let current_manifest_path = canonicalize_or_original(cargo_toml_path.to_path_buf()).await;
    let workspace_manifest =
        if preload_workspace_manifest || !requests.workspace_dependencies.is_empty() {
            load_workspace_document_tree_async(document_tree, cargo_toml_path, toml_version).await
        } else {
            None
        };

    let (path_dependencies, workspace_dependencies) = tokio::join!(
        load_local_manifest_entries(cargo_toml_path, requests.path_dependencies, toml_version),
        async {
            match workspace_manifest.as_ref() {
                Some((workspace_cargo_toml_path, workspace_document_tree)) => {
                    load_workspace_manifest_entries(
                        workspace_document_tree,
                        workspace_cargo_toml_path,
                        requests.workspace_dependencies,
                        toml_version,
                    )
                    .await
                }
                None => Vec::new(),
            }
        }
    );

    let mut local_manifest_cache = LocalManifestCache::default();
    if let Some((workspace_cargo_toml_path, workspace_document_tree)) = workspace_manifest {
        local_manifest_cache.workspace_manifests.insert(
            current_manifest_path,
            Some(workspace_cargo_toml_path.clone()),
        );
        local_manifest_cache
            .manifests
            .insert(workspace_cargo_toml_path, workspace_document_tree);
    }

    for manifest in path_dependencies.into_iter().chain(workspace_dependencies) {
        local_manifest_cache
            .manifests
            .insert(manifest.cargo_toml_path, manifest.document_tree);
    }

    local_manifest_cache
}

fn collect_local_manifest_requests(
    document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
    dependency_version_enabled: bool,
    default_features_enabled: bool,
) -> LocalManifestRequests {
    let mut requests = LocalManifestRequests::default();

    for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        collect_local_manifest_requests_from_keys(
            document_tree,
            &[dependency_key],
            visible_range,
            dependency_version_enabled,
            default_features_enabled,
            &mut requests,
        );
    }

    collect_local_manifest_requests_from_keys(
        document_tree,
        &["workspace", "dependencies"],
        visible_range,
        dependency_version_enabled,
        default_features_enabled,
        &mut requests,
    );

    if let Some((_, Value::Table(targets))) = dig_keys(document_tree, &["target"]) {
        for (target_key, target_value) in targets.key_values() {
            let Value::Table(_) = target_value else {
                continue;
            };

            for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                collect_local_manifest_requests_from_keys(
                    document_tree,
                    &["target", target_key.value.as_str(), dependency_key],
                    visible_range,
                    dependency_version_enabled,
                    default_features_enabled,
                    &mut requests,
                );
            }
        }
    }

    requests
}

fn collect_local_manifest_requests_from_keys(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_keys: &[&str],
    visible_range: tombi_text::Range,
    dependency_version_enabled: bool,
    default_features_enabled: bool,
    requests: &mut LocalManifestRequests,
) {
    let Some((_, Value::Table(dependencies))) = dig_keys(document_tree, dependency_keys) else {
        return;
    };

    for (dependency_key, dependency_value) in dependencies.key_values() {
        let Value::Table(table) = dependency_value else {
            continue;
        };
        if !needs_visible_local_manifest_prefetch(
            dependency_key.value.as_str(),
            dependency_value,
            visible_range,
            dependency_version_enabled,
            default_features_enabled,
        ) {
            continue;
        }

        if let Some(Value::String(path)) = table.get("path") {
            requests.path_dependencies.insert(path.value().to_string());
        }

        if let Some(Value::Boolean(workspace)) = table.get("workspace")
            && workspace.value()
        {
            requests
                .workspace_dependencies
                .insert(dependency_key.value.to_string());
        }
    }
}

fn needs_visible_local_manifest_prefetch(
    dependency_key: &str,
    dependency_value: &Value,
    visible_range: tombi_text::Range,
    dependency_version_enabled: bool,
    default_features_enabled: bool,
) -> bool {
    if dependency_version_enabled
        && dependency_version_hint(dependency_key, dependency_value).is_some_and(|hint| {
            hint.local_version_source.is_some()
                && tombi_text::Range::at(hint.position).intersects(visible_range)
        })
    {
        return true;
    }

    default_features_enabled
        && local_default_features_request_position(dependency_value)
            .is_some_and(|position| tombi_text::Range::at(position).intersects(visible_range))
}

fn local_default_features_request_position(
    dependency_value: &Value,
) -> Option<tombi_text::Position> {
    let Value::Table(table) = dependency_value else {
        return None;
    };
    if dependency_table_default_features_disabled(table) {
        return None;
    }

    let Some(Value::Array(features)) = table.get("features") else {
        return None;
    };
    let is_local_dependency = table.get("path").is_some()
        || matches!(table.get("workspace"), Some(Value::Boolean(workspace)) if workspace.value());

    is_local_dependency.then_some(features.range().end)
}

async fn load_local_manifest_entries(
    base_manifest_path: &Path,
    dependency_paths: HashSet<String>,
    toml_version: TomlVersion,
) -> Vec<LocalManifestData> {
    let base_manifest_path = base_manifest_path.to_path_buf();

    stream::iter(dependency_paths.into_iter().map(|dependency_path| {
        let base_manifest_path = base_manifest_path.clone();
        async move {
            load_manifest_data_for_dependency_path(
                &base_manifest_path,
                &dependency_path,
                toml_version,
            )
            .await
        }
    }))
    .buffer_unordered(LOCAL_MANIFEST_PREFETCH_CONCURRENCY)
    .filter_map(|entry| async move { entry })
    .collect()
    .await
}

async fn load_workspace_manifest_entries(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace_cargo_toml_path: &Path,
    dependency_keys: HashSet<String>,
    toml_version: TomlVersion,
) -> Vec<LocalManifestData> {
    if dependency_keys.is_empty() {
        return Vec::new();
    }

    let workspace_requests = dependency_keys
        .into_iter()
        .filter_map(|dependency_key| {
            let (_, workspace_dependency_value) = dig_keys(
                &workspace_document_tree,
                &["workspace", "dependencies", dependency_key.as_str()],
            )?;
            let Value::Table(workspace_dependency_table) = workspace_dependency_value else {
                return None;
            };
            let Some(Value::String(path)) = workspace_dependency_table.get("path") else {
                return None;
            };
            Some(path.value().to_string())
        })
        .collect::<Vec<_>>();

    stream::iter(workspace_requests.into_iter().map(|dependency_path| {
        let workspace_cargo_toml_path = workspace_cargo_toml_path.to_path_buf();
        async move {
            load_manifest_data_for_dependency_path(
                &workspace_cargo_toml_path,
                &dependency_path,
                toml_version,
            )
            .await
        }
    }))
    .buffer_unordered(LOCAL_MANIFEST_PREFETCH_CONCURRENCY)
    .filter_map(|entry| async move { entry })
    .collect()
    .await
}

async fn load_workspace_document_tree_async(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<(PathBuf, tombi_document_tree::DocumentTree)> {
    if document_tree.contains_key("workspace") {
        return Some((cargo_toml_path.to_path_buf(), document_tree.clone()));
    }

    if let Some(workspace_path) = get_workspace_path(document_tree) {
        let workspace_cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
            cargo_toml_path,
            Path::new(workspace_path),
            "Cargo.toml",
        )?;
        let workspace_cargo_toml_path = canonicalize_or_original(workspace_cargo_toml_path).await;
        let workspace_document_tree =
            load_cargo_toml_async(&workspace_cargo_toml_path, toml_version).await?;

        return workspace_document_tree
            .contains_key("workspace")
            .then_some((workspace_cargo_toml_path, workspace_document_tree));
    }

    let (workspace_cargo_toml_path, workspace_document_tree) =
        tombi_extension_manifest::find_ancestor_manifest_async(
            cargo_toml_path,
            "Cargo.toml",
            |path| async move { load_cargo_toml_async(&path, toml_version).await },
            |tree| tree.contains_key("workspace"),
        )
        .await?;
    let workspace_cargo_toml_path = canonicalize_or_original(workspace_cargo_toml_path).await;

    Some((workspace_cargo_toml_path, workspace_document_tree))
}

async fn load_manifest_data_for_dependency_path(
    base_manifest_path: &Path,
    dependency_path: &str,
    toml_version: TomlVersion,
) -> Option<LocalManifestData> {
    let cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
        base_manifest_path,
        Path::new(dependency_path),
        "Cargo.toml",
    )?;
    let cargo_toml_path = canonicalize_or_original(cargo_toml_path).await;
    let document_tree = load_cargo_toml_async(&cargo_toml_path, toml_version).await?;

    Some(LocalManifestData {
        cargo_toml_path,
        document_tree,
    })
}

async fn canonicalize_or_original(path: PathBuf) -> PathBuf {
    match tokio::fs::canonicalize(&path).await {
        Ok(path) => path,
        Err(_) => path,
    }
}

async fn load_cargo_toml_async(
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<tombi_document_tree::DocumentTree> {
    let toml_text = tokio::fs::read_to_string(cargo_toml_path).await.ok()?;

    tokio::task::spawn_blocking(move || {
        let root = tombi_ast::Root::cast(tombi_parser::parse(&toml_text).into_syntax_node())?;
        root.try_into_document_tree(toml_version).ok()
    })
    .await
    .ok()
    .flatten()
}

fn canonicalize_or_original_sync(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

fn load_cached_manifest_document_tree(
    local_manifest_cache: &mut LocalManifestCache,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<(PathBuf, tombi_document_tree::DocumentTree)> {
    let canonicalized_path = canonicalize_or_original_sync(cargo_toml_path.to_path_buf());
    if let Some(document_tree) = local_manifest_cache.manifests.get(&canonicalized_path) {
        return Some((canonicalized_path, document_tree.clone()));
    }

    let (_, document_tree) = load_cargo_toml(&canonicalized_path, toml_version)?;
    local_manifest_cache
        .manifests
        .insert(canonicalized_path.clone(), document_tree.clone());

    Some((canonicalized_path, document_tree))
}

fn load_local_dependency_document_tree_cached(
    local_manifest_cache: &mut LocalManifestCache,
    cargo_toml_path: &Path,
    dependency_path: &str,
    toml_version: TomlVersion,
) -> Option<(PathBuf, tombi_document_tree::DocumentTree)> {
    let dependency_cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
        cargo_toml_path,
        Path::new(dependency_path),
        "Cargo.toml",
    )?;

    load_cached_manifest_document_tree(
        local_manifest_cache,
        &dependency_cargo_toml_path,
        toml_version,
    )
}

fn find_workspace_cargo_toml_cached(
    local_manifest_cache: &mut LocalManifestCache,
    cargo_toml_path: &Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
) -> Option<(PathBuf, tombi_document_tree::DocumentTree)> {
    let cache_key = canonicalize_or_original_sync(cargo_toml_path.to_path_buf());

    if let Some(cached_workspace_path) = local_manifest_cache.workspace_manifests.get(&cache_key) {
        let workspace_cargo_toml_path = cached_workspace_path.clone()?;
        return load_cached_manifest_document_tree(
            local_manifest_cache,
            &workspace_cargo_toml_path,
            toml_version,
        );
    }

    let workspace_manifest = if let Some(workspace_path) = workspace_path {
        let workspace_cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
            cargo_toml_path,
            Path::new(workspace_path),
            "Cargo.toml",
        )?;
        let (workspace_cargo_toml_path, workspace_document_tree) =
            load_cached_manifest_document_tree(
                local_manifest_cache,
                &workspace_cargo_toml_path,
                toml_version,
            )?;

        workspace_document_tree
            .contains_key("workspace")
            .then_some((workspace_cargo_toml_path, workspace_document_tree))
    } else {
        let mut current_dir = cargo_toml_path.parent()?;

        let mut workspace_manifest = None;
        while let Some(target_dir) = current_dir.parent() {
            current_dir = target_dir;
            let workspace_cargo_toml_path = current_dir.join("Cargo.toml");

            if !workspace_cargo_toml_path.is_file() {
                continue;
            }

            let (workspace_cargo_toml_path, workspace_document_tree) =
                load_cached_manifest_document_tree(
                    local_manifest_cache,
                    &workspace_cargo_toml_path,
                    toml_version,
                )?;

            if workspace_document_tree.contains_key("workspace") {
                workspace_manifest = Some((workspace_cargo_toml_path, workspace_document_tree));
                break;
            }
        }

        workspace_manifest
    };

    local_manifest_cache.workspace_manifests.insert(
        cache_key,
        workspace_manifest
            .as_ref()
            .map(|(workspace_cargo_toml_path, _)| workspace_cargo_toml_path.clone()),
    );

    workspace_manifest
}

async fn registry_default_features_inlay_hints(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Vec<InlayHint>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(Vec::new());
    }

    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(Vec::new());
    };
    let cargo_lock = load_cached_cargo_lock(&cargo_toml_path, toml_version).await;

    let mut hints = Vec::new();

    for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        collect_registry_default_features_inlay_hints(
            document_tree,
            &[dependency_key],
            &cargo_toml_path,
            cargo_lock.as_ref(),
            toml_version,
            visible_range,
            offline,
            cache_options,
            &mut hints,
        )
        .await?;
    }

    collect_registry_default_features_inlay_hints(
        document_tree,
        &["workspace", "dependencies"],
        &cargo_toml_path,
        cargo_lock.as_ref(),
        toml_version,
        visible_range,
        offline,
        cache_options,
        &mut hints,
    )
    .await?;

    if let Some((_, Value::Table(targets))) = dig_keys(document_tree, &["target"]) {
        for (target_key, target_value) in targets.key_values() {
            let Value::Table(_) = target_value else {
                continue;
            };

            for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                collect_registry_default_features_inlay_hints(
                    document_tree,
                    &["target", target_key.value.as_str(), dependency_key],
                    &cargo_toml_path,
                    cargo_lock.as_ref(),
                    toml_version,
                    visible_range,
                    offline,
                    cache_options,
                    &mut hints,
                )
                .await?;
            }
        }
    }

    Ok(hints)
}

async fn collect_registry_default_features_inlay_hints(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_keys: &[&str],
    cargo_toml_path: &Path,
    cargo_lock: Option<&CargoLock>,
    toml_version: TomlVersion,
    visible_range: tombi_text::Range,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    hints: &mut Vec<InlayHint>,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    let Some((_, Value::Table(dependencies))) = dig_keys(document_tree, dependency_keys) else {
        return Ok(());
    };

    for (dependency_key, dependency_value) in dependencies.key_values() {
        let Some(hint) = registry_dependency_default_features_hint(
            document_tree,
            dependency_key.value.as_str(),
            dependency_value,
            cargo_toml_path,
            cargo_lock,
            toml_version,
            offline,
            cache_options,
        )
        .await?
        else {
            continue;
        };

        if !tombi_text::Range::at(hint.position).intersects(visible_range) {
            continue;
        }

        hints.push(InlayHint {
            position: hint.position,
            label: hint.label,
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(hint.tooltip),
            padding_left: Some(true),
            padding_right: Some(false),
        });
    }

    Ok(())
}

async fn registry_dependency_default_features_hint(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_key: &str,
    dependency_value: &Value,
    cargo_toml_path: &Path,
    cargo_lock: Option<&CargoLock>,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<DefaultFeaturesHint>, tower_lsp::jsonrpc::Error> {
    let Value::Table(table) = dependency_value else {
        return Ok(None);
    };

    let Some(Value::Array(features)) = table.get("features") else {
        return Ok(None);
    };

    if dependency_table_default_features_disabled(table) || table.get("path").is_some() {
        return Ok(None);
    }

    let registry_dependency = if let Some(Value::String(version)) = table.get("version") {
        Some((
            dependency_package_name(dependency_key, dependency_value).to_string(),
            version.value().to_string(),
        ))
    } else if matches!(table.get("workspace"), Some(Value::Boolean(workspace)) if workspace.value())
    {
        let Some((_, _, workspace_document_tree)) = find_workspace_cargo_toml(
            cargo_toml_path,
            get_workspace_path(document_tree),
            toml_version,
        ) else {
            return Ok(None);
        };
        let Some((_, workspace_dependency_value)) = dig_keys(
            &workspace_document_tree,
            &["workspace", "dependencies", dependency_key],
        ) else {
            return Ok(None);
        };
        let Value::Table(workspace_dependency_table) = workspace_dependency_value else {
            return Ok(None);
        };

        if dependency_table_default_features_disabled(workspace_dependency_table)
            || workspace_dependency_table.get("path").is_some()
        {
            return Ok(None);
        }

        let Some(Value::String(version)) = workspace_dependency_table.get("version") else {
            return Ok(None);
        };

        Some((
            dependency_package_name(dependency_key, workspace_dependency_value).to_string(),
            version.value().to_string(),
        ))
    } else {
        None
    };

    let Some((crate_name, version)) = registry_dependency else {
        return Ok(None);
    };
    let Some(version) = cargo_lock
        .and_then(|lock| lock.resolve_dependency_version(&crate_name, &version))
        .or_else(|| crate::cargo_lock::exact_crates_io_version(&version))
    else {
        return Ok(None);
    };

    let Some(crate_features) =
        fetch_registry_crate_features(&crate_name, &version, offline, cache_options).await
    else {
        return Ok(None);
    };
    let Some(default_features) = crate_features.get("default") else {
        return Ok(None);
    };

    Ok(build_default_features_hint(
        features.range().end,
        default_features.clone(),
        &collect_feature_names(features),
    ))
}

async fn fetch_registry_crate_features(
    crate_name: &str,
    version: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<HashMap<String, Vec<String>>> {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}/{version}");
    let resp =
        fetch_cached_remote_json::<CratesIoVersionDetailResponse>(&url, offline, cache_options)
            .await?;
    Some(resp.version.features)
}

fn collect_feature_names(features: &tombi_document_tree::Array) -> HashSet<String> {
    features
        .values()
        .iter()
        .filter_map(|feature| match feature {
            Value::String(feature) => Some(feature.value().to_string()),
            _ => None,
        })
        .collect()
}

fn dependency_table_default_features_disabled(table: &tombi_document_tree::Table) -> bool {
    table
        .get("default-features")
        .is_some_and(|value| match value {
            Value::Boolean(boolean) => !boolean.value(),
            _ => false,
        })
}

fn build_default_features_hint(
    position: tombi_text::Position,
    mut default_features: Vec<String>,
    existing_features: &HashSet<String>,
) -> Option<DefaultFeaturesHint> {
    default_features.sort();

    let missing_default_features = default_features
        .iter()
        .filter(|feature| !existing_features.contains(feature.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    if missing_default_features.is_empty() {
        return None;
    }

    Some(DefaultFeaturesHint {
        position,
        label: format_default_features_label(&missing_default_features),
        tooltip: format_default_features_tooltip(&default_features),
    })
}

fn format_default_features_label(default_features: &[String]) -> String {
    format!(
        " + [{}]",
        default_features
            .iter()
            .map(|feature| format!("{feature:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn format_default_features_tooltip(default_features: &[String]) -> String {
    format!(
        "Default Features:\n{}",
        default_features
            .iter()
            .map(|feature| format!("- {feature:?}"))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn package_default_features(
    dependency_document_tree: &tombi_document_tree::DocumentTree,
) -> Option<Vec<String>> {
    let (_, Value::Array(default_features)) =
        dig_keys(dependency_document_tree, &["features", "default"])?
    else {
        return None;
    };

    let default_features = default_features
        .values()
        .iter()
        .filter_map(|value| match value {
            Value::String(feature) => Some(feature.value().to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();

    (!default_features.is_empty()).then_some(default_features)
}

fn cargo_lock_dependency_version(
    cargo_lock_cache: &CargoLockInlayCacheData,
    dependency_keys: &[&str],
    dependency_name: &str,
    current_package: Option<&CurrentPackage<'_>>,
    workspace_member_packages: Option<&[WorkspaceMemberPackage]>,
) -> Option<String> {
    if dependency_keys == ["workspace", "dependencies"] {
        return workspace_dependency_lock_version(
            cargo_lock_cache,
            workspace_member_packages,
            dependency_name,
        );
    }

    let current_package = current_package?;
    cargo_lock_cache.resolved_dependency_version(
        current_package.name,
        &current_package.version,
        dependency_name,
    )
}

fn workspace_dependency_lock_version(
    cargo_lock_cache: &CargoLockInlayCacheData,
    workspace_member_packages: Option<&[WorkspaceMemberPackage]>,
    dependency_name: &str,
) -> Option<String> {
    let workspace_member_packages = workspace_member_packages?;

    let resolved_versions = workspace_member_packages
        .iter()
        .filter_map(|package| {
            cargo_lock_cache.resolved_dependency_version(
                &package.name,
                &package.version,
                dependency_name,
            )
        })
        .collect::<HashSet<_>>();

    (resolved_versions.len() == 1)
        .then(|| resolved_versions.into_iter().next())
        .flatten()
}

fn workspace_dependency_lock_versions<'a>(
    cargo_lock_cache: &CargoLockInlayCacheData,
    workspace_member_packages: Option<&[WorkspaceMemberPackage]>,
    dependency_names: impl Iterator<Item = &'a str>,
) -> HashMap<String, String> {
    let Some(workspace_member_packages) = workspace_member_packages else {
        return HashMap::new();
    };

    let dependency_names = dependency_names.map(str::to_string).collect::<HashSet<_>>();
    if dependency_names.is_empty() {
        return HashMap::new();
    }

    let mut resolved_versions = dependency_names
        .into_iter()
        .map(|dependency_name| (dependency_name, HashSet::new()))
        .collect::<HashMap<_, _>>();

    for package in workspace_member_packages {
        for (dependency_name, versions) in &mut resolved_versions {
            let Some(resolved_version) = cargo_lock_cache.resolved_dependency_version(
                &package.name,
                &package.version,
                dependency_name,
            ) else {
                continue;
            };
            versions.insert(resolved_version);
        }
    }

    resolved_versions
        .into_iter()
        .filter_map(|(dependency_name, versions)| {
            (versions.len() == 1)
                .then(|| {
                    versions
                        .into_iter()
                        .next()
                        .map(|version| (dependency_name, version))
                })
                .flatten()
        })
        .collect()
}

fn workspace_member_packages(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
) -> Option<Vec<WorkspaceMemberPackage>> {
    if document_tree.contains_key("workspace") {
        return workspace_member_packages_for_workspace(
            document_tree,
            cargo_toml_path,
            local_manifest_cache,
            toml_version,
        );
    }

    let (workspace_cargo_toml_path, workspace_document_tree) = find_workspace_cargo_toml_cached(
        local_manifest_cache,
        cargo_toml_path,
        get_workspace_path(document_tree),
        toml_version,
    )?;

    workspace_member_packages_for_workspace(
        &workspace_document_tree,
        &workspace_cargo_toml_path,
        local_manifest_cache,
        toml_version,
    )
}

fn workspace_document_tree<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
) -> Option<WorkspaceDocumentTree<'a>> {
    if document_tree.contains_key("workspace") {
        return Some(WorkspaceDocumentTree::Current(document_tree));
    }

    let (_, workspace_document_tree) = find_workspace_cargo_toml_cached(
        local_manifest_cache,
        cargo_toml_path,
        get_workspace_path(document_tree),
        toml_version,
    )?;

    Some(WorkspaceDocumentTree::External(workspace_document_tree))
}

fn workspace_member_packages_for_workspace(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace_cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
) -> Option<Vec<WorkspaceMemberPackage>> {
    let member_patterns = workspace_member_patterns(workspace_document_tree);
    if member_patterns.is_empty() {
        return None;
    }

    let workspace_cargo_toml_path =
        canonicalize_or_original_sync(workspace_cargo_toml_path.to_path_buf());
    if let Some(workspace_member_packages) = local_manifest_cache
        .workspace_member_packages
        .get(&workspace_cargo_toml_path)
    {
        return Some(workspace_member_packages.clone());
    }

    let exclude_patterns = extract_exclude_patterns(workspace_document_tree);
    let workspace_dir_path = workspace_cargo_toml_path.parent()?;
    let mut workspace_member_packages = Vec::new();

    for (_, manifest_path) in
        find_package_cargo_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let (manifest_path, member_document_tree) =
            load_cached_manifest_document_tree(local_manifest_cache, &manifest_path, toml_version)?;
        let package_name = current_package_name(&member_document_tree)?;
        let package_version = package_version(
            &member_document_tree,
            &manifest_path,
            local_manifest_cache,
            toml_version,
        )?;

        workspace_member_packages.push(WorkspaceMemberPackage {
            name: package_name.to_string(),
            version: package_version,
        });
    }

    local_manifest_cache
        .workspace_member_packages
        .insert(workspace_cargo_toml_path, workspace_member_packages.clone());

    Some(workspace_member_packages)
}

fn current_package<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
) -> Option<CurrentPackage<'a>> {
    Some(CurrentPackage {
        name: current_package_name(document_tree)?,
        version: package_version(
            document_tree,
            cargo_toml_path,
            local_manifest_cache,
            toml_version,
        )?,
    })
}

fn workspace_member_patterns(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
) -> Vec<&tombi_document_tree::String> {
    match dig_keys(workspace_document_tree, &["workspace", "members"]) {
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

async fn load_cargo_lock_cache(
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<CargoLockInlayCacheData> {
    let cargo_lock_path = find_cargo_lock_path(cargo_toml_path)?;
    let cache_key = cargo_lock_cache_key(&cargo_lock_path);
    let cache_version = file_cache_version(&cargo_lock_path);

    let cache_value = get_or_load_json(&cache_key, cache_version, {
        let cargo_lock_path = cargo_lock_path.clone();
        move || async move { load_cargo_lock_cache_json(cargo_lock_path, toml_version).await }
    })
    .await?;

    CargoLockInlayCacheData::deserialize(cache_value.as_ref()).ok()
}

async fn load_cargo_lock_cache_json(
    cargo_lock_path: PathBuf,
    toml_version: TomlVersion,
) -> Option<serde_json::Value> {
    tokio::task::spawn_blocking(move || parse_cargo_lock_cache_json(&cargo_lock_path, toml_version))
        .await
        .ok()
        .flatten()
}

fn parse_cargo_lock_cache_json(
    cargo_lock_path: &Path,
    toml_version: TomlVersion,
) -> Option<serde_json::Value> {
    let cargo_lock = load_cargo_lock_from_path(cargo_lock_path, toml_version)?;
    serde_json::to_value(cargo_lock.into_inlay_cache_data()).ok()
}

fn cargo_lock_cache_key(cargo_lock_path: &Path) -> String {
    format!(
        "{CARGO_EXTENSION_ID}:{INLAY_HINT_LOCKFILE_KEY}:{}",
        cargo_lock_path.display()
    )
}

fn package_version(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    local_manifest_cache: &mut LocalManifestCache,
    toml_version: TomlVersion,
) -> Option<String> {
    let (_, package_version) = dig_keys(document_tree, &["package", "version"])?;

    match package_version {
        Value::String(version) => Some(version.value().to_string()),
        Value::Table(table) => {
            let Some(Value::Boolean(workspace)) = table.get("workspace") else {
                return None;
            };
            if !workspace.value() {
                return None;
            }

            if document_tree.contains_key("workspace") {
                let (_, Value::String(version)) =
                    dig_keys(document_tree, &["workspace", "package", "version"])?
                else {
                    return None;
                };

                return Some(version.value().to_string());
            }

            let (_, workspace_document_tree) = find_workspace_cargo_toml_cached(
                local_manifest_cache,
                cargo_toml_path,
                get_workspace_path(document_tree),
                toml_version,
            )?;
            let (_, Value::String(version)) = dig_keys(
                &workspace_document_tree,
                &["workspace", "package", "version"],
            )?
            else {
                return None;
            };

            Some(version.value().to_string())
        }
        _ => None,
    }
}

fn current_package_name(document_tree: &tombi_document_tree::DocumentTree) -> Option<&str> {
    let (_, Value::String(package_name)) = dig_keys(document_tree, &["package", "name"])? else {
        return None;
    };

    Some(package_name.value())
}

fn version_hint_label(
    current_version: Option<&str>,
    resolved_version: &str,
    always_show: bool,
) -> Option<String> {
    if !always_show && current_version == Some(resolved_version) {
        return None;
    }

    Some(format!(r#" → "{resolved_version}""#))
}

fn workspace_value_hint_label(value: &Value) -> Option<String> {
    if matches!(value, Value::Incomplete { .. }) {
        return None;
    }

    Some(format!(" → {}", sanitize_value_for_hint(value)))
}

fn sanitize_value_for_hint(value: &Value) -> String {
    let sanitized = value.to_string().replace('\r', "\\r").replace('\n', "\\n");
    let value_len = sanitized.chars().count();
    if value_len <= MAX_WORKSPACE_VALUE_HINT_CHARS {
        return sanitized;
    }

    let truncated_len = MAX_WORKSPACE_VALUE_HINT_CHARS.saturating_sub(3);
    let mut truncated = sanitized.chars().take(truncated_len).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn cargo_inlay_hint_root_enabled(features: Option<&tombi_config::CargoExtensionFeatures>) -> bool {
    features.map_or(
        true,
        tombi_config::CargoExtensionFeatures::inlay_hint_enabled,
    )
}

fn cargo_inlay_hint_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
    feature: CargoInlayHintFeature,
) -> bool {
    match feature {
        CargoInlayHintFeature::DependencyVersion => features.map_or(
            true,
            tombi_config::CargoExtensionFeatures::dependency_version_inlay_hint_enabled,
        ),
        CargoInlayHintFeature::DefaultFeatures => features.map_or(
            true,
            tombi_config::CargoExtensionFeatures::default_features_inlay_hint_enabled,
        ),
        CargoInlayHintFeature::WorkspaceValue => features.map_or(
            true,
            tombi_config::CargoExtensionFeatures::workspace_value_inlay_hint_enabled,
        ),
    }
}

impl CargoLock {
    fn into_inlay_cache_data(self) -> CargoLockInlayCacheData {
        let unique_package_versions = self.unique_package_versions().clone();
        let mut crates = HashMap::new();

        for package in &self.packages {
            crates
                .entry(CrateName::new(&package.name))
                .or_insert_with(HashMap::new)
                .insert(
                    CrateVersion::new(&package.version),
                    package.resolved_dependencies(&unique_package_versions),
                );
        }

        CargoLockInlayCacheData { crates }
    }
}

impl CargoLockPackage {
    fn resolved_dependencies(
        &self,
        unique_package_versions: &HashMap<String, Option<String>>,
    ) -> CrateResolvedDependencies {
        let dependency_names = self
            .dependencies
            .iter()
            .map(|dependency| dependency.name.clone())
            .collect::<HashSet<_>>();

        let by_dependency = dependency_names
            .into_iter()
            .filter_map(|dependency_name| {
                let resolved_version = self.lockfile_resolved_dependency_version(
                    &dependency_name,
                    unique_package_versions,
                )?;
                Some((
                    DependencyCrateName::new(&dependency_name),
                    ResolvedDependencyVersion::new(resolved_version),
                ))
            })
            .collect();

        CrateResolvedDependencies { by_dependency }
    }
}

impl CrateName {
    fn new(crate_name: &str) -> Self {
        Self(crate_name.to_string())
    }
}

impl Borrow<str> for CrateName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl CrateVersion {
    fn new(crate_version: &str) -> Self {
        Self(crate_version.to_string())
    }
}

impl Borrow<str> for CrateVersion {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl DependencyCrateName {
    fn new(dependency_name: &str) -> Self {
        Self(dependency_name.to_string())
    }
}

impl Borrow<str> for DependencyCrateName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl ResolvedDependencyVersion {
    fn new(version: String) -> Self {
        Self { version }
    }
}

impl CargoLockInlayCacheData {
    fn resolved_dependency_version(
        &self,
        crate_name: &str,
        crate_version: &str,
        dependency_name: &str,
    ) -> Option<String> {
        self.crates
            .get(crate_name)
            .and_then(|versions| versions.get(crate_version))
            .and_then(|dependencies| dependencies.by_dependency.get(dependency_name))
            .map(|resolved| resolved.version.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::cargo_lock::{CargoLockDependency, CargoLockPackage};
    use tombi_ast::AstNode;
    use tombi_document_tree::TryIntoDocumentTree;

    fn parse_document_tree(source: &str) -> tombi_document_tree::DocumentTree {
        let root = tombi_ast::Root::cast(tombi_parser::parse(source).into_syntax_node())
            .expect("expected root");
        root.try_into_document_tree(TomlVersion::default())
            .expect("expected document tree")
    }

    fn parse_value(source: &str) -> Value {
        let document_tree = parse_document_tree(source);
        let (_, value) = dig_keys(&document_tree, &["value"]).expect("expected value");
        value.clone()
    }

    #[test]
    fn collects_local_manifest_requests_only_for_visible_path_dependency_hints() {
        let document_tree = parse_document_tree(
            r#"
            [dependencies]
            serde = { path = "../serde" }
            tokio = { path = "../tokio" }
            "#,
        );
        let (_, serde_value) =
            dig_keys(&document_tree, &["dependencies", "serde"]).expect("expected serde");
        let visible_range = tombi_text::Range::at(
            dependency_version_hint("serde", serde_value)
                .unwrap()
                .position,
        );

        let requests = collect_local_manifest_requests(&document_tree, visible_range, true, false);

        assert_eq!(requests.path_dependencies.len(), 1);
        assert!(requests.path_dependencies.contains("../serde"));
        assert!(!requests.path_dependencies.contains("../tokio"));
        assert!(requests.workspace_dependencies.is_empty());
    }

    #[test]
    fn collects_local_manifest_requests_for_visible_workspace_default_feature_hints() {
        let document_tree = parse_document_tree(
            r#"
            [dependencies]
            serde = { workspace = true, features = ["derive"] }
            tokio = { workspace = true, features = ["rt"] }
            "#,
        );
        let (_, serde_value) =
            dig_keys(&document_tree, &["dependencies", "serde"]).expect("expected serde");
        let visible_range = tombi_text::Range::at(
            local_default_features_request_position(serde_value)
                .expect("expected feature position"),
        );

        let requests = collect_local_manifest_requests(&document_tree, visible_range, false, true);

        assert!(requests.path_dependencies.is_empty());
        assert_eq!(requests.workspace_dependencies.len(), 1);
        assert!(requests.workspace_dependencies.contains("serde"));
        assert!(!requests.workspace_dependencies.contains("tokio"));
    }

    #[test]
    fn skips_local_manifest_requests_when_local_hints_are_offscreen() {
        let document_tree = parse_document_tree(
            r#"
            [dependencies]
            serde = { path = "../serde" }
            "#,
        );

        let requests = collect_local_manifest_requests(
            &document_tree,
            tombi_text::Range::at(tombi_text::Position::new(0, 0)),
            true,
            true,
        );

        assert!(requests.path_dependencies.is_empty());
        assert!(requests.workspace_dependencies.is_empty());
    }

    #[test]
    fn adds_plain_version_label() {
        assert_eq!(
            version_hint_label(Some("0.15.6"), "0.15.8", false),
            Some(r#" → "0.15.8""#.to_string())
        );
    }

    #[test]
    fn omits_hint_when_version_is_already_resolved() {
        assert_eq!(version_hint_label(Some("0.15.8"), "0.15.8", false), None);
    }

    #[test]
    fn keeps_plain_version_when_current_version_is_missing() {
        assert_eq!(
            version_hint_label(None, "0.15.8", false),
            Some(r#" → "0.15.8""#.to_string())
        );
    }

    #[test]
    fn keeps_hint_for_workspace_inheritance_even_when_versions_match() {
        assert_eq!(
            version_hint_label(Some("0.15.8"), "0.15.8", true),
            Some(r#" → "0.15.8""#.to_string())
        );
    }

    #[test]
    fn renders_workspace_value_hint_label() {
        let value = parse_value(r#"value = ["tombi", "cargo"]"#);

        assert_eq!(
            workspace_value_hint_label(&value),
            Some(r#" → ["tombi", "cargo"]"#.to_string())
        );
    }

    #[test]
    fn normalizes_workspace_value_hint_label_to_single_line() {
        let value = parse_value(
            r#"
            value = """
            hello
            world
            """
            "#,
        );

        let label = workspace_value_hint_label(&value).expect("expected label");
        assert!(!label.contains('\n'));
        assert!(label.contains("\\n"));
    }

    #[test]
    fn truncates_workspace_value_hint_label() {
        let value = parse_value(
            r#"value = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz""#,
        );

        let label = workspace_value_hint_label(&value).expect("expected label");
        assert!(label.ends_with("..."));
        assert!(label.chars().count() <= MAX_WORKSPACE_VALUE_HINT_CHARS + 3);
    }

    #[test]
    fn keeps_workspace_package_hints_when_dependency_version_hints_are_disabled() {
        let temp_dir = tempfile::tempdir().expect("expected temp dir");
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        fs::write(
            &cargo_toml_path,
            r#"
            [package]
            name = "app"
            version = { workspace = true }

            [workspace]
            members = ["."]

            [workspace.package]
            version = "0.0.0-dev"
            "#,
        )
        .expect("expected Cargo.toml");

        let (_, document_tree) =
            load_cargo_toml(&cargo_toml_path, TomlVersion::default()).expect("expected manifest");
        let uri = tombi_uri::Uri::from_file_path(&cargo_toml_path).expect("expected uri");
        let features = tombi_config::CargoExtensionFeatures::Features(
            tombi_config::CargoExtensionFeatureTree {
                lsp: Some(tombi_config::CargoLspFeatures::Features(
                    tombi_config::CargoLspFeatureTree {
                        inlay_hint: Some(tombi_config::CargoInlayHintFeatures::Features(
                            tombi_config::CargoInlayHintFeatureTree {
                                dependency_version: Some(tombi_config::ToggleFeature {
                                    enabled: Some(false.into()),
                                }),
                                default_features: None,
                                workspace_value: None,
                            },
                        )),
                        ..Default::default()
                    },
                )),
            },
        );

        let hints = inlay_hint_impl(
            &uri,
            &document_tree,
            tombi_text::Range::new(
                tombi_text::Position::new(0, 0),
                tombi_text::Position::new(8, 0),
            ),
            None,
            LocalManifestCache::default(),
            TomlVersion::default(),
            Some(&features),
        )
        .expect("expected inlay hint result");

        assert_eq!(
            hints,
            Some(vec![InlayHint {
                position: tombi_text::Position::new(3, 40),
                label: r#" → "0.0.0-dev""#.to_string(),
                kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
                tooltip: Some(WORKSPACE_INHERITED_VALUE_TOOLTIP.to_string()),
                padding_left: Some(true),
                padding_right: Some(false),
            }])
        );
    }

    #[test]
    fn disables_workspace_package_hints_when_workspace_inlay_hints_are_disabled() {
        let temp_dir = tempfile::tempdir().expect("expected temp dir");
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        fs::write(
            &cargo_toml_path,
            r#"
            [package]
            name = "app"
            version = { workspace = true }

            [workspace]
            members = ["."]

            [workspace.package]
            version = "0.0.0-dev"
            "#,
        )
        .expect("expected Cargo.toml");

        let (_, document_tree) =
            load_cargo_toml(&cargo_toml_path, TomlVersion::default()).expect("expected manifest");
        let uri = tombi_uri::Uri::from_file_path(&cargo_toml_path).expect("expected uri");
        let features = tombi_config::CargoExtensionFeatures::Features(
            tombi_config::CargoExtensionFeatureTree {
                lsp: Some(tombi_config::CargoLspFeatures::Features(
                    tombi_config::CargoLspFeatureTree {
                        inlay_hint: Some(tombi_config::CargoInlayHintFeatures::Features(
                            tombi_config::CargoInlayHintFeatureTree {
                                dependency_version: None,
                                default_features: None,
                                workspace_value: Some(tombi_config::ToggleFeature {
                                    enabled: Some(false.into()),
                                }),
                            },
                        )),
                        ..Default::default()
                    },
                )),
            },
        );

        let hints = inlay_hint_impl(
            &uri,
            &document_tree,
            tombi_text::Range::new(
                tombi_text::Position::new(0, 0),
                tombi_text::Position::new(8, 0),
            ),
            None,
            LocalManifestCache::default(),
            TomlVersion::default(),
            Some(&features),
        )
        .expect("expected inlay hint result");

        assert_eq!(hints, None);
    }

    #[test]
    fn cargo_lock_cache_data_resolves_dependency_version() {
        let cargo_lock = CargoLock::new(vec![
            CargoLockPackage {
                name: "demo".to_string(),
                version: "0.1.0".to_string(),
                dependencies: vec![CargoLockDependency {
                    name: "serde".to_string(),
                    version: Some("1.0.228".to_string()),
                }],
            },
            CargoLockPackage {
                name: "serde".to_string(),
                version: "1.0.228".to_string(),
                dependencies: Vec::new(),
            },
        ]);

        let cache_data = cargo_lock.into_inlay_cache_data();

        assert_eq!(
            cache_data.resolved_dependency_version("demo", "0.1.0", "serde"),
            Some("1.0.228".to_string())
        );
    }

    #[test]
    fn cargo_lock_cache_data_survives_json_roundtrip() {
        let cargo_lock = CargoLock::new(vec![
            CargoLockPackage {
                name: "demo".to_string(),
                version: "0.1.0".to_string(),
                dependencies: vec![CargoLockDependency {
                    name: "tokio".to_string(),
                    version: Some("1.47.1".to_string()),
                }],
            },
            CargoLockPackage {
                name: "tokio".to_string(),
                version: "1.47.1".to_string(),
                dependencies: Vec::new(),
            },
        ]);

        let cache_data = cargo_lock.into_inlay_cache_data();
        let roundtrip = serde_json::from_value::<CargoLockInlayCacheData>(
            serde_json::to_value(&cache_data).expect("expected json value"),
        )
        .expect("expected cache data");

        assert_eq!(
            roundtrip.resolved_dependency_version("demo", "0.1.0", "tokio"),
            Some("1.47.1".to_string())
        );
    }

    #[test]
    fn resolve_dependency_version_prefers_exact_or_lockfile_version() {
        let cargo_lock = CargoLock::new(vec![
            CargoLockPackage {
                name: "demo".to_string(),
                version: "0.1.0".to_string(),
                dependencies: vec![CargoLockDependency {
                    name: "criterion".to_string(),
                    version: Some("0.5.1".to_string()),
                }],
            },
            CargoLockPackage {
                name: "criterion".to_string(),
                version: "0.5.1".to_string(),
                dependencies: Vec::new(),
            },
        ]);

        assert_eq!(
            cargo_lock.resolve_dependency_version("criterion", "=0.5.1"),
            Some("0.5.1".to_string())
        );
        assert_eq!(
            cargo_lock.resolve_dependency_version("criterion", "0.5"),
            Some("0.5.1".to_string())
        );
    }
}
