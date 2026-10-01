use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
};

use futures::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use tombi_config::TomlVersion;
use tombi_document_snapshot::DocumentSnapshot;
use tombi_document_tree_syntax::{Value, dig_keys};
use tombi_extension::{
    InlayHint, InlayHintKind, fetch_cached_remote_json, file_cache_version, get_or_load_json,
};
use tombi_hashmap::{HashMap, HashSet};

use crate::{
    cargo_lock::{
        CARGO_EXTENSION_ID, CargoLock, CargoLockPackage, find_cargo_lock_path,
        load_cached_cargo_lock, load_cargo_lock_from_path,
    },
    cargo_toml::load_cargo_toml_snapshot,
    crates_io::CratesIoVersionDetailResponse,
    dependency_package_name, find_workspace_cargo_toml, get_workspace_cargo_toml_path,
    workspace::{extract_exclude_patterns, find_package_cargo_toml_paths},
};

const RESOLVED_VERSION_TOOLTIP: &str = "Resolved version in Cargo.lock";
const LOCAL_PATH_VERSION_TOOLTIP: &str = "Version from local dependency Cargo.toml";
const WORKSPACE_INHERITED_VALUE_TOOLTIP: &str = "Inherited value from workspace";
const MAX_WORKSPACE_VALUE_HINT_CHARS: usize = 80;
const INLAY_HINT_LOCKFILE_KEY: &str = "inlay_hint.lockfile";
const LOCAL_CARGO_TOML_PREFETCH_CONCURRENCY: usize = 8;
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
    offset: tombi_text::Offset,
    current_version: Option<String>,
    always_show: bool,
    local_version_source: Option<LocalVersionSource>,
}

enum LocalVersionSource {
    Path(String),
    WorkspaceDependency(String),
}

struct DefaultFeaturesHint {
    offset: tombi_text::Offset,
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

/// The `Cargo.toml` files that were read while collecting the hints of one request.
///
/// Each file is parsed once, and every hint that needs it reuses the snapshot.
#[derive(Default)]
struct LocalCargoTomlCache {
    cargo_tomls: HashMap<PathBuf, DocumentSnapshot>,
    workspace_cargo_tomls: HashMap<PathBuf, Option<PathBuf>>,
    workspace_member_packages: HashMap<PathBuf, Vec<WorkspaceMemberPackage>>,
}

struct LocalCargoTomlData {
    cargo_toml_path: PathBuf,
    snapshot: DocumentSnapshot,
}

#[derive(Default)]
struct LocalCargoTomlRequests {
    path_dependencies: HashSet<String>,
    workspace_dependencies: HashSet<String>,
}

enum WorkspaceCargoToml {
    /// The document of the request is the workspace `Cargo.toml`.
    Current,
    External {
        cargo_toml_path: PathBuf,
        snapshot: DocumentSnapshot,
    },
}

pub async fn inlay_hint(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    visible_span: tombi_text::Span,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<InlayHint>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }

    if !features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.inlay_hint())
        .map(|inlay_hint| inlay_hint.enabled())
        .unwrap_or_default()
        .value()
    {
        return Ok(None);
    }

    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    let dependency_version_enabled =
        cargo_inlay_hint_enabled(features, CargoInlayHintFeature::DependencyVersion);
    let default_features_enabled =
        cargo_inlay_hint_enabled(features, CargoInlayHintFeature::DefaultFeatures);
    let (cargo_lock_cache, local_cargo_toml_cache) = tokio::join!(
        async {
            if dependency_version_enabled {
                load_cargo_lock_cache(&cargo_toml_path, toml_version).await
            } else {
                None
            }
        },
        async {
            if dependency_version_enabled || default_features_enabled {
                preload_local_cargo_toml_cache(
                    document_tree,
                    &cargo_toml_path,
                    visible_span,
                    toml_version,
                    dependency_version_enabled,
                    default_features_enabled,
                    has_visible_workspace_value_targets(document_tree, visible_span),
                )
                .await
            } else {
                LocalCargoTomlCache::default()
            }
        }
    );

    let mut hints = inlay_hint_impl(
        text_document_uri,
        document_tree,
        visible_span,
        cargo_lock_cache,
        local_cargo_toml_cache,
        toml_version,
        features,
    )?
    .unwrap_or_default();

    if features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.inlay_hint())
        .and_then(|inlay_hint| inlay_hint.default_features())
        .map(|default_features| default_features.enabled())
        .unwrap_or_default()
        .value()
    {
        hints.extend(
            registry_default_features_inlay_hints(
                text_document_uri,
                document_tree,
                visible_span,
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
        hints.sort_by_key(|hint| hint.offset);
        Ok(Some(hints))
    }
}

fn inlay_hint_impl(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    visible_span: tombi_text::Span,
    cargo_lock_cache: Option<CargoLockInlayCacheData>,
    mut local_cargo_toml_cache: LocalCargoTomlCache,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<InlayHint>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
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
            &mut local_cargo_toml_cache,
            toml_version,
            visible_span,
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
                &mut local_cargo_toml_cache,
                toml_version,
                visible_span,
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
            &mut local_cargo_toml_cache,
            toml_version,
            visible_span,
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
                        &["target", target_key.value(), dependency_key],
                        &cargo_toml_path,
                        cargo_lock_cache.as_ref(),
                        &mut local_cargo_toml_cache,
                        toml_version,
                        visible_span,
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
        hints.sort_by_key(|hint| hint.offset);
        Ok(Some(hints))
    }
}

fn collect_workspace_value_inlay_hints(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    toml_version: TomlVersion,
    visible_span: tombi_text::Span,
    hints: &mut Vec<InlayHint>,
) {
    if !has_visible_workspace_value_targets(document_tree, visible_span) {
        return;
    }

    with_workspace_document_tree(
        document_tree,
        cargo_toml_path,
        local_cargo_toml_cache,
        toml_version,
        |_, _, workspace_document_tree| {
            collect_workspace_package_inlay_hints(
                document_tree,
                workspace_document_tree,
                visible_span,
                hints,
            );
            collect_workspace_lints_inlay_hints(
                document_tree,
                workspace_document_tree,
                visible_span,
                hints,
            );
        },
    );
}

fn collect_workspace_package_inlay_hints(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    visible_span: tombi_text::Span,
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

        push_workspace_value_hint(workspace, workspace_value, visible_span, hints);
    }
}

fn collect_workspace_lints_inlay_hints(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    visible_span: tombi_text::Span,
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

    push_workspace_value_hint(workspace, workspace_value, visible_span, hints);
}

fn has_visible_workspace_value_targets(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    visible_span: tombi_text::Span,
) -> bool {
    WORKSPACE_PACKAGE_ITEMS.iter().any(|package_item| {
        matches!(
            dig_keys(document_tree, &["package", package_item, "workspace"]),
            Some((_, Value::Boolean(workspace)))
                if workspace.value()
                    && tombi_text::Span::empty(workspace.span().end).intersects(visible_span)
        )
    }) || matches!(
        dig_keys(document_tree, &["lints", "workspace"]),
        Some((_, Value::Boolean(workspace)))
            if workspace.value()
                && tombi_text::Span::empty(workspace.span().end).intersects(visible_span)
    )
}

fn push_workspace_value_hint(
    workspace: &tombi_document_tree_syntax::Boolean,
    workspace_value: &Value<'_>,
    visible_span: tombi_text::Span,
    hints: &mut Vec<InlayHint>,
) {
    if !tombi_text::Span::empty(workspace.span().end).intersects(visible_span) {
        return;
    }

    let Some(label) = workspace_value_hint_label(workspace_value) else {
        return;
    };

    hints.push(InlayHint {
        offset: workspace.span().end,
        label,
        kind: Some(InlayHintKind::TYPE),
        tooltip: Some(WORKSPACE_INHERITED_VALUE_TOOLTIP.to_string()),
        padding_left: Some(true),
        padding_right: Some(false),
    });
}

fn collect_dependency_inlay_hints(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    dependency_keys: &[&str],
    cargo_toml_path: &Path,
    cargo_lock_cache: Option<&CargoLockInlayCacheData>,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    toml_version: TomlVersion,
    visible_span: tombi_text::Span,
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
                dependency_version_hint(dependency_key.value(), dependency_value)
            && tombi_text::Span::empty(version_hint.offset).intersects(visible_span)
        {
            version_hints.push(version_hint);
        }

        if default_features_enabled
            && let Some(default_features_hint) = dependency_default_features_hint(
                document_tree,
                dependency_key.value(),
                dependency_value,
                cargo_toml_path,
                local_cargo_toml_cache,
                toml_version,
            )
            && tombi_text::Span::empty(default_features_hint.offset).intersects(visible_span)
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
                local_cargo_toml_cache,
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
                local_cargo_toml_cache,
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
            offset,
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
                local_cargo_toml_cache,
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
            offset,
            label,
            kind: Some(InlayHintKind::TYPE),
            tooltip: Some(tooltip.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        });
    }

    hints.extend(default_feature_hints.into_iter().map(|hint| InlayHint {
        offset: hint.offset,
        label: hint.label,
        kind: Some(InlayHintKind::TYPE),
        tooltip: Some(hint.tooltip),
        padding_left: Some(true),
        padding_right: Some(false),
    }));
}
fn dependency_version_hint(
    dependency_key: &str,
    dependency_value: &Value<'_>,
) -> Option<DependencyVersionHint> {
    let dependency_name = dependency_package_name(dependency_key, dependency_value).to_string();

    match dependency_value {
        Value::String(version) => Some(DependencyVersionHint {
            dependency_name,
            offset: version.span().end,
            current_version: Some(version.value().to_string()),
            always_show: false,
            local_version_source: None,
        }),
        Value::Table(table) => {
            if let Some(Value::String(version)) = table.get("version") {
                return Some(DependencyVersionHint {
                    dependency_name,
                    offset: version.span().end,
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
                    offset: workspace.span().end,
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
                    offset: path.span().end,
                    current_version: None,
                    always_show: false,
                    local_version_source: Some(LocalVersionSource::Path(path.value().to_string())),
                });
            }

            if let Some(Value::String(git)) = table.get("git") {
                return Some(DependencyVersionHint {
                    dependency_name,
                    offset: git.span().end,
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
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    source: Option<&LocalVersionSource>,
    cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    toml_version: TomlVersion,
) -> Option<String> {
    match source? {
        LocalVersionSource::Path(path) => with_local_dependency_cargo_toml(
            local_cargo_toml_cache,
            cargo_toml_path,
            path,
            toml_version,
            |local_cargo_toml_cache, dependency_cargo_toml_path, dependency_document_tree| {
                package_version(
                    dependency_document_tree,
                    dependency_cargo_toml_path,
                    local_cargo_toml_cache,
                    toml_version,
                )
            },
        )
        .flatten(),
        LocalVersionSource::WorkspaceDependency(dependency_key) => {
            workspace_path_dependency_version(
                document_tree,
                dependency_key,
                cargo_toml_path,
                local_cargo_toml_cache,
                toml_version,
            )
        }
    }
}

fn workspace_path_dependency_version(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    dependency_key: &str,
    cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    toml_version: TomlVersion,
) -> Option<String> {
    with_workspace_document_tree(
        document_tree,
        cargo_toml_path,
        local_cargo_toml_cache,
        toml_version,
        |local_cargo_toml_cache, workspace_cargo_toml_path, workspace_document_tree| {
            let (_, workspace_dependency_value) = dig_keys(
                workspace_document_tree,
                &["workspace", "dependencies", dependency_key],
            )?;
            let Value::Table(workspace_dependency_table) = workspace_dependency_value else {
                return None;
            };
            let Some(Value::String(path)) = workspace_dependency_table.get("path") else {
                return None;
            };

            with_local_dependency_cargo_toml(
                local_cargo_toml_cache,
                workspace_cargo_toml_path,
                path.value(),
                toml_version,
                |local_cargo_toml_cache, dependency_cargo_toml_path, dependency_document_tree| {
                    package_version(
                        dependency_document_tree,
                        dependency_cargo_toml_path,
                        local_cargo_toml_cache,
                        toml_version,
                    )
                },
            )
            .flatten()
        },
    )
    .flatten()
}

fn dependency_default_features_hint(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    dependency_key: &str,
    dependency_value: &Value<'_>,
    cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
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
        local_cargo_toml_cache,
        toml_version,
    )?;

    build_default_features_hint(
        features.span().end,
        default_features,
        &collect_feature_names(features),
    )
}

fn dependency_default_features(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    dependency_key: &str,
    dependency_value: &Value<'_>,
    cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    toml_version: TomlVersion,
) -> Option<Vec<String>> {
    let Value::Table(table) = dependency_value else {
        return None;
    };

    if let Some(Value::String(path)) = table.get("path") {
        return with_local_dependency_cargo_toml(
            local_cargo_toml_cache,
            cargo_toml_path,
            path.value(),
            toml_version,
            |_, _, dependency_document_tree| package_default_features(dependency_document_tree),
        )
        .flatten();
    }

    let Some(Value::Boolean(workspace)) = table.get("workspace") else {
        return None;
    };
    if !workspace.value() {
        return None;
    }

    with_workspace_document_tree(
        document_tree,
        cargo_toml_path,
        local_cargo_toml_cache,
        toml_version,
        |local_cargo_toml_cache, workspace_cargo_toml_path, workspace_document_tree| {
            let (_, workspace_dependency_value) = dig_keys(
                workspace_document_tree,
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

            with_local_dependency_cargo_toml(
                local_cargo_toml_cache,
                workspace_cargo_toml_path,
                path.value(),
                toml_version,
                |_, _, dependency_document_tree| package_default_features(dependency_document_tree),
            )
            .flatten()
        },
    )
    .flatten()
}

async fn preload_local_cargo_toml_cache(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cargo_toml_path: &Path,
    visible_span: tombi_text::Span,
    toml_version: TomlVersion,
    dependency_version_enabled: bool,
    default_features_enabled: bool,
    preload_workspace_cargo_toml: bool,
) -> LocalCargoTomlCache {
    let requests = collect_local_cargo_toml_requests(
        document_tree,
        visible_span,
        dependency_version_enabled,
        default_features_enabled,
    );
    let current_cargo_toml_path = canonicalize_or_original(cargo_toml_path.to_path_buf());
    let workspace_cargo_toml =
        if preload_workspace_cargo_toml || !requests.workspace_dependencies.is_empty() {
            load_workspace_cargo_toml_async(document_tree, cargo_toml_path, toml_version).await
        } else {
            None
        };

    let (workspace_base_cargo_toml_path, workspace_dependency_paths) = match &workspace_cargo_toml {
        Some(WorkspaceCargoToml::Current) => (
            cargo_toml_path.to_path_buf(),
            workspace_dependency_request_paths(document_tree, &requests.workspace_dependencies),
        ),
        Some(WorkspaceCargoToml::External {
            cargo_toml_path,
            snapshot,
        }) => (
            cargo_toml_path.clone(),
            workspace_dependency_request_paths(
                snapshot.document_tree(),
                &requests.workspace_dependencies,
            ),
        ),
        None => (PathBuf::new(), Vec::new()),
    };

    let (path_dependencies, workspace_dependencies) = tokio::join!(
        load_local_cargo_toml_entries(cargo_toml_path, requests.path_dependencies, toml_version),
        load_local_cargo_toml_entries(
            &workspace_base_cargo_toml_path,
            workspace_dependency_paths,
            toml_version
        )
    );

    let mut local_cargo_toml_cache = LocalCargoTomlCache::default();
    if let Some(WorkspaceCargoToml::External {
        cargo_toml_path: workspace_cargo_toml_path,
        snapshot,
    }) = workspace_cargo_toml
    {
        local_cargo_toml_cache.workspace_cargo_tomls.insert(
            current_cargo_toml_path,
            Some(workspace_cargo_toml_path.clone()),
        );
        local_cargo_toml_cache
            .cargo_tomls
            .insert(workspace_cargo_toml_path, snapshot);
    }

    for cargo_toml in path_dependencies.into_iter().chain(workspace_dependencies) {
        local_cargo_toml_cache
            .cargo_tomls
            .insert(cargo_toml.cargo_toml_path, cargo_toml.snapshot);
    }

    local_cargo_toml_cache
}

fn collect_local_cargo_toml_requests(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    visible_span: tombi_text::Span,
    dependency_version_enabled: bool,
    default_features_enabled: bool,
) -> LocalCargoTomlRequests {
    let mut requests = LocalCargoTomlRequests::default();

    for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        collect_local_cargo_toml_requests_from_keys(
            document_tree,
            &[dependency_key],
            visible_span,
            dependency_version_enabled,
            default_features_enabled,
            &mut requests,
        );
    }

    collect_local_cargo_toml_requests_from_keys(
        document_tree,
        &["workspace", "dependencies"],
        visible_span,
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
                collect_local_cargo_toml_requests_from_keys(
                    document_tree,
                    &["target", target_key.value(), dependency_key],
                    visible_span,
                    dependency_version_enabled,
                    default_features_enabled,
                    &mut requests,
                );
            }
        }
    }

    requests
}

fn collect_local_cargo_toml_requests_from_keys(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    dependency_keys: &[&str],
    visible_span: tombi_text::Span,
    dependency_version_enabled: bool,
    default_features_enabled: bool,
    requests: &mut LocalCargoTomlRequests,
) {
    let Some((_, Value::Table(dependencies))) = dig_keys(document_tree, dependency_keys) else {
        return;
    };

    for (dependency_key, dependency_value) in dependencies.key_values() {
        let Value::Table(table) = dependency_value else {
            continue;
        };
        if !needs_visible_local_cargo_toml_prefetch(
            dependency_key.value(),
            dependency_value,
            visible_span,
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
                .insert(dependency_key.value().to_string());
        }
    }
}

fn needs_visible_local_cargo_toml_prefetch(
    dependency_key: &str,
    dependency_value: &Value<'_>,
    visible_span: tombi_text::Span,
    dependency_version_enabled: bool,
    default_features_enabled: bool,
) -> bool {
    if dependency_version_enabled
        && dependency_version_hint(dependency_key, dependency_value).is_some_and(|hint| {
            hint.local_version_source.is_some()
                && tombi_text::Span::empty(hint.offset).intersects(visible_span)
        })
    {
        return true;
    }

    default_features_enabled
        && local_default_features_request_offset(dependency_value)
            .is_some_and(|offset| tombi_text::Span::empty(offset).intersects(visible_span))
}

fn local_default_features_request_offset(
    dependency_value: &Value<'_>,
) -> Option<tombi_text::Offset> {
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

    is_local_dependency.then_some(features.span().end)
}

async fn load_local_cargo_toml_entries(
    base_cargo_toml_path: &Path,
    dependency_paths: impl IntoIterator<Item = String>,
    toml_version: TomlVersion,
) -> Vec<LocalCargoTomlData> {
    let base_cargo_toml_path = base_cargo_toml_path.to_path_buf();

    stream::iter(dependency_paths.into_iter().map(|dependency_path| {
        let base_cargo_toml_path = base_cargo_toml_path.clone();
        async move {
            load_cargo_toml_data_for_dependency_path(
                &base_cargo_toml_path,
                &dependency_path,
                toml_version,
            )
            .await
        }
    }))
    .buffer_unordered(LOCAL_CARGO_TOML_PREFETCH_CONCURRENCY)
    .filter_map(|entry| async move { entry })
    .collect()
    .await
}

fn workspace_dependency_request_paths(
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    dependency_keys: &HashSet<String>,
) -> Vec<String> {
    dependency_keys
        .iter()
        .filter_map(|dependency_key| {
            let (_, workspace_dependency_value) = dig_keys(
                workspace_document_tree,
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
        .collect()
}

async fn load_workspace_cargo_toml_async(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<WorkspaceCargoToml> {
    if document_tree.contains_key("workspace") {
        return Some(WorkspaceCargoToml::Current);
    }

    if let Some(workspace_path) = get_workspace_cargo_toml_path(document_tree) {
        let workspace_cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
            cargo_toml_path,
            Path::new(workspace_path),
            "Cargo.toml",
        )?;
        let workspace_cargo_toml_path = canonicalize_or_original(workspace_cargo_toml_path);
        let snapshot = load_cargo_toml_async(&workspace_cargo_toml_path, toml_version).await?;

        return snapshot
            .document_tree()
            .contains_key("workspace")
            .then_some(WorkspaceCargoToml::External {
                cargo_toml_path: workspace_cargo_toml_path,
                snapshot,
            });
    }

    let (workspace_cargo_toml_path, snapshot) =
        tombi_extension_manifest::find_ancestor_manifest_async(
            cargo_toml_path,
            "Cargo.toml",
            |path| async move { load_cargo_toml_async(&path, toml_version).await },
            |snapshot| snapshot.document_tree().contains_key("workspace"),
        )
        .await?;

    Some(WorkspaceCargoToml::External {
        cargo_toml_path: canonicalize_or_original(workspace_cargo_toml_path),
        snapshot,
    })
}

async fn load_cargo_toml_data_for_dependency_path(
    base_cargo_toml_path: &Path,
    dependency_path: &str,
    toml_version: TomlVersion,
) -> Option<LocalCargoTomlData> {
    let cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
        base_cargo_toml_path,
        Path::new(dependency_path),
        "Cargo.toml",
    )?;
    let cargo_toml_path = canonicalize_or_original(cargo_toml_path);
    let snapshot = load_cargo_toml_async(&cargo_toml_path, toml_version).await?;

    Some(LocalCargoTomlData {
        cargo_toml_path,
        snapshot,
    })
}

fn canonicalize_or_original(path: PathBuf) -> PathBuf {
    match tombi_fs::canonicalize(&path) {
        Ok(path) => path,
        Err(_) => path,
    }
}

/// Reads and parses `cargo_toml_path`. It is `None` when the file is not a valid manifest.
async fn load_cargo_toml_async(
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<DocumentSnapshot> {
    let toml_text = tombi_fs::read_to_string_async(cargo_toml_path).await.ok()?;

    tombi_fs::run_blocking(move || {
        let snapshot = DocumentSnapshot::parse(toml_text, toml_version);

        snapshot
            .document_tree_errors()
            .is_empty()
            .then_some(snapshot)
    })
    .await
    .ok()
    .flatten()
}

fn canonicalize_or_original_sync(path: PathBuf) -> PathBuf {
    tombi_fs::canonicalize(&path).unwrap_or(path)
}

/// Runs `f` on the `Cargo.toml` at `cargo_toml_path`, reading it once per request.
fn with_cached_cargo_toml<R>(
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
    f: impl FnOnce(&mut LocalCargoTomlCache, &Path, &tombi_document_tree_syntax::DocumentTree<'_>) -> R,
) -> Option<R> {
    let canonicalized_path = canonicalize_or_original_sync(cargo_toml_path.to_path_buf());
    let snapshot = match local_cargo_toml_cache.cargo_tomls.get(&canonicalized_path) {
        Some(snapshot) => snapshot.clone(),
        None => {
            let snapshot = load_cargo_toml_snapshot(&canonicalized_path, toml_version)?;
            local_cargo_toml_cache
                .cargo_tomls
                .insert(canonicalized_path.clone(), snapshot.clone());
            snapshot
        }
    };

    Some(f(
        local_cargo_toml_cache,
        &canonicalized_path,
        snapshot.document_tree(),
    ))
}

fn with_local_dependency_cargo_toml<R>(
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    cargo_toml_path: &Path,
    dependency_path: &str,
    toml_version: TomlVersion,
    f: impl FnOnce(&mut LocalCargoTomlCache, &Path, &tombi_document_tree_syntax::DocumentTree<'_>) -> R,
) -> Option<R> {
    let dependency_cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
        cargo_toml_path,
        Path::new(dependency_path),
        "Cargo.toml",
    )?;

    with_cached_cargo_toml(
        local_cargo_toml_cache,
        &dependency_cargo_toml_path,
        toml_version,
        f,
    )
}

/// Runs `f` on the workspace `Cargo.toml` of the document.
///
/// The document itself is used when it is the workspace.
fn with_workspace_document_tree<R>(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    toml_version: TomlVersion,
    f: impl FnOnce(&mut LocalCargoTomlCache, &Path, &tombi_document_tree_syntax::DocumentTree<'_>) -> R,
) -> Option<R> {
    if document_tree.contains_key("workspace") {
        return Some(f(local_cargo_toml_cache, cargo_toml_path, document_tree));
    }

    let workspace_cargo_toml_path = find_workspace_cargo_toml_path_with_local_cache(
        local_cargo_toml_cache,
        cargo_toml_path,
        get_workspace_cargo_toml_path(document_tree),
        toml_version,
    )?;

    with_cached_cargo_toml(
        local_cargo_toml_cache,
        &workspace_cargo_toml_path,
        toml_version,
        f,
    )
}

// Inlay hints keep a short-lived local cache to avoid rereading Cargo.toml
// files while collecting multiple hints for the same request.
fn find_workspace_cargo_toml_path_with_local_cache(
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    cargo_toml_path: &Path,
    workspace_path: Option<&str>,
    toml_version: TomlVersion,
) -> Option<PathBuf> {
    let cache_key = canonicalize_or_original_sync(cargo_toml_path.to_path_buf());

    if let Some(cached_workspace_path) =
        local_cargo_toml_cache.workspace_cargo_tomls.get(&cache_key)
    {
        return cached_workspace_path.clone();
    }

    let workspace_cargo_toml_path = if let Some(workspace_path) = workspace_path {
        let workspace_cargo_toml_path = tombi_extension_manifest::resolve_manifest_path(
            cargo_toml_path,
            Path::new(workspace_path),
            "Cargo.toml",
        )?;

        with_cached_cargo_toml(
            local_cargo_toml_cache,
            &workspace_cargo_toml_path,
            toml_version,
            |_, workspace_cargo_toml_path, workspace_document_tree| {
                workspace_document_tree
                    .contains_key("workspace")
                    .then(|| workspace_cargo_toml_path.to_path_buf())
            },
        )?
    } else {
        let mut current_dir = cargo_toml_path.parent()?;

        let mut workspace_cargo_toml_path = None;
        while let Some(target_dir) = current_dir.parent() {
            current_dir = target_dir;
            let candidate_cargo_toml_path = current_dir.join("Cargo.toml");

            if !tombi_fs::is_file(&candidate_cargo_toml_path) {
                continue;
            }

            let candidate_workspace_cargo_toml_path = with_cached_cargo_toml(
                local_cargo_toml_cache,
                &candidate_cargo_toml_path,
                toml_version,
                |_, candidate_cargo_toml_path, candidate_document_tree| {
                    candidate_document_tree
                        .contains_key("workspace")
                        .then(|| candidate_cargo_toml_path.to_path_buf())
                },
            )?;

            if candidate_workspace_cargo_toml_path.is_some() {
                workspace_cargo_toml_path = candidate_workspace_cargo_toml_path;
                break;
            }
        }

        workspace_cargo_toml_path
    };

    local_cargo_toml_cache
        .workspace_cargo_tomls
        .insert(cache_key, workspace_cargo_toml_path.clone());

    workspace_cargo_toml_path
}

async fn registry_default_features_inlay_hints(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    visible_span: tombi_text::Span,
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
            visible_span,
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
        visible_span,
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
                    &["target", target_key.value(), dependency_key],
                    &cargo_toml_path,
                    cargo_lock.as_ref(),
                    toml_version,
                    visible_span,
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
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    dependency_keys: &[&str],
    cargo_toml_path: &Path,
    cargo_lock: Option<&CargoLock>,
    toml_version: TomlVersion,
    visible_span: tombi_text::Span,
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
            dependency_key.value(),
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

        if !tombi_text::Span::empty(hint.offset).intersects(visible_span) {
            continue;
        }

        hints.push(InlayHint {
            offset: hint.offset,
            label: hint.label,
            kind: Some(InlayHintKind::TYPE),
            tooltip: Some(hint.tooltip),
            padding_left: Some(true),
            padding_right: Some(false),
        });
    }

    Ok(())
}

async fn registry_dependency_default_features_hint(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    dependency_key: &str,
    dependency_value: &Value<'_>,
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
        let Some(workspace_registry_dependency) = find_workspace_cargo_toml(
            cargo_toml_path,
            get_workspace_cargo_toml_path(document_tree),
            toml_version,
            |_, workspace_document_tree, _| {
                let (_, workspace_dependency_value) = dig_keys(
                    workspace_document_tree,
                    &["workspace", "dependencies", dependency_key],
                )?;
                let Value::Table(workspace_dependency_table) = workspace_dependency_value else {
                    return None;
                };

                if dependency_table_default_features_disabled(workspace_dependency_table)
                    || workspace_dependency_table.get("path").is_some()
                {
                    return None;
                }

                let Some(Value::String(version)) = workspace_dependency_table.get("version") else {
                    return None;
                };

                Some((
                    dependency_package_name(dependency_key, workspace_dependency_value).to_string(),
                    version.value().to_string(),
                ))
            },
        )
        .flatten() else {
            return Ok(None);
        };

        Some(workspace_registry_dependency)
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

    let Some(mut crate_features) =
        fetch_registry_crate_features(&crate_name, &version, offline, cache_options).await
    else {
        return Ok(None);
    };
    let Some(default_features) = crate_features.remove("default") else {
        return Ok(None);
    };

    Ok(build_default_features_hint(
        features.span().end,
        default_features,
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

fn collect_feature_names(features: &tombi_document_tree_syntax::Array<'_>) -> HashSet<String> {
    features
        .values()
        .iter()
        .filter_map(|feature| match feature {
            Value::String(feature) => Some(feature.value().to_string()),
            _ => None,
        })
        .collect()
}

fn dependency_table_default_features_disabled(
    table: &tombi_document_tree_syntax::Table<'_>,
) -> bool {
    table
        .get("default-features")
        .is_some_and(|value| match value {
            Value::Boolean(boolean) => !boolean.value(),
            _ => false,
        })
}

fn build_default_features_hint(
    offset: tombi_text::Offset,
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
        offset,
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
    dependency_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
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
    cargo_lock_cache
        .resolved_dependency_version(
            current_package.name,
            &current_package.version,
            dependency_name,
        )
        .map(str::to_string)
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
        .map(str::to_string)
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

    let mut resolved_versions: HashMap<String, HashSet<&str>> = dependency_names
        .into_iter()
        .map(|dependency_name| (dependency_name, HashSet::new()))
        .collect();

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
                        .map(|version| (dependency_name, version.to_string()))
                })
                .flatten()
        })
        .collect()
}

fn workspace_member_packages(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    toml_version: TomlVersion,
) -> Option<Vec<WorkspaceMemberPackage>> {
    with_workspace_document_tree(
        document_tree,
        cargo_toml_path,
        local_cargo_toml_cache,
        toml_version,
        |local_cargo_toml_cache, workspace_cargo_toml_path, workspace_document_tree| {
            workspace_member_packages_for_workspace(
                workspace_document_tree,
                workspace_cargo_toml_path,
                local_cargo_toml_cache,
                toml_version,
            )
        },
    )
    .flatten()
}

fn workspace_member_packages_for_workspace(
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    workspace_cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    toml_version: TomlVersion,
) -> Option<Vec<WorkspaceMemberPackage>> {
    let member_patterns = workspace_member_patterns(workspace_document_tree);
    if member_patterns.is_empty() {
        return None;
    }

    let workspace_cargo_toml_path =
        canonicalize_or_original_sync(workspace_cargo_toml_path.to_path_buf());
    if let Some(workspace_member_packages) = local_cargo_toml_cache
        .workspace_member_packages
        .get(&workspace_cargo_toml_path)
    {
        return Some(workspace_member_packages.clone());
    }

    let exclude_patterns = extract_exclude_patterns(workspace_document_tree);
    let workspace_dir_path = workspace_cargo_toml_path.parent()?;
    let mut workspace_member_packages = Vec::new();

    for (_, cargo_toml_path) in
        find_package_cargo_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let member_package = with_cached_cargo_toml(
            local_cargo_toml_cache,
            &cargo_toml_path,
            toml_version,
            |local_cargo_toml_cache, cargo_toml_path, member_document_tree| {
                let package_name = current_package_name(member_document_tree)?.to_string();
                let package_version = package_version(
                    member_document_tree,
                    cargo_toml_path,
                    local_cargo_toml_cache,
                    toml_version,
                )?;

                Some(WorkspaceMemberPackage {
                    name: package_name,
                    version: package_version,
                })
            },
        )
        .flatten()?;

        workspace_member_packages.push(member_package);
    }

    local_cargo_toml_cache
        .workspace_member_packages
        .insert(workspace_cargo_toml_path, workspace_member_packages.clone());

    Some(workspace_member_packages)
}

fn current_package<'a>(
    document_tree: &'a tombi_document_tree_syntax::DocumentTree<'_>,
    cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
    toml_version: TomlVersion,
) -> Option<CurrentPackage<'a>> {
    Some(CurrentPackage {
        name: current_package_name(document_tree)?,
        version: package_version(
            document_tree,
            cargo_toml_path,
            local_cargo_toml_cache,
            toml_version,
        )?,
    })
}

fn workspace_member_patterns<'a, 't>(
    workspace_document_tree: &'a tombi_document_tree_syntax::DocumentTree<'t>,
) -> Vec<&'a tombi_document_tree_syntax::String<'t>> {
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
    tombi_fs::run_blocking(move || parse_cargo_lock_cache_json(&cargo_lock_path, toml_version))
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
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    cargo_toml_path: &Path,
    local_cargo_toml_cache: &mut LocalCargoTomlCache,
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

            with_workspace_document_tree(
                document_tree,
                cargo_toml_path,
                local_cargo_toml_cache,
                toml_version,
                |_, _, workspace_document_tree| {
                    let (_, Value::String(version)) = dig_keys(
                        workspace_document_tree,
                        &["workspace", "package", "version"],
                    )?
                    else {
                        return None;
                    };

                    Some(version.value().to_string())
                },
            )
            .flatten()
        }
        _ => None,
    }
}

fn current_package_name<'a>(
    document_tree: &'a tombi_document_tree_syntax::DocumentTree<'_>,
) -> Option<&'a str> {
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

fn workspace_value_hint_label(value: &Value<'_>) -> Option<String> {
    if matches!(value, Value::Incomplete { .. }) {
        return None;
    }

    Some(format!(" → {}", sanitize_value_for_hint(value)))
}

fn sanitize_value_for_hint(value: &Value<'_>) -> String {
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

fn cargo_inlay_hint_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
    feature: CargoInlayHintFeature,
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.inlay_hint())
        .and_then(|inlay_hint| match feature {
            CargoInlayHintFeature::DependencyVersion => inlay_hint.dependency_version(),
            CargoInlayHintFeature::DefaultFeatures => inlay_hint.default_features(),
            CargoInlayHintFeature::WorkspaceValue => inlay_hint.workspace_value(),
        })
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}

impl CargoLock {
    fn into_inlay_cache_data(self) -> CargoLockInlayCacheData {
        let (packages, unique_package_versions) = self.into_parts();
        let mut crates = HashMap::new();

        for package in &packages {
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
            .map(|dependency| dependency.name.as_str())
            .collect::<HashSet<_>>();

        let by_dependency = dependency_names
            .into_iter()
            .filter_map(|dependency_name| {
                let resolved_version = self.lockfile_resolved_dependency_version(
                    dependency_name,
                    unique_package_versions,
                )?;
                Some((
                    DependencyCrateName::new(dependency_name),
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
    fn new(version: &str) -> Self {
        Self {
            version: version.to_string(),
        }
    }
}

impl CargoLockInlayCacheData {
    fn resolved_dependency_version(
        &self,
        crate_name: &str,
        crate_version: &str,
        dependency_name: &str,
    ) -> Option<&str> {
        self.crates
            .get(crate_name)
            .and_then(|versions| versions.get(crate_version))
            .and_then(|dependencies| dependencies.by_dependency.get(dependency_name))
            .map(|resolved| resolved.version.as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::cargo_lock::{CargoLockDependency, CargoLockPackage};
    use crate::cargo_toml::with_cargo_toml_text;
    use tombi_text::EncodingKind;

    fn with_document_tree<R>(
        source: &str,
        f: impl FnOnce(&tombi_document_tree_syntax::DocumentTree<'_>) -> R,
    ) -> R {
        with_cargo_toml_text(source, TomlVersion::default(), |document_tree, _| {
            f(document_tree)
        })
        .expect("expected document tree")
    }

    fn with_value<R>(source: &str, f: impl FnOnce(&Value<'_>) -> R) -> R {
        with_document_tree(source, |document_tree| {
            let (_, value) = dig_keys(document_tree, &["value"]).expect("expected value");
            f(value)
        })
    }

    #[test]
    fn collects_local_cargo_toml_requests_only_for_visible_path_dependency_hints() {
        with_document_tree(
            r#"
            [dependencies]
            serde = { path = "../serde" }
            tokio = { path = "../tokio" }
            "#,
            |document_tree| {
                let (_, serde_value) =
                    dig_keys(document_tree, &["dependencies", "serde"]).expect("expected serde");
                let visible_span = tombi_text::Span::empty(
                    dependency_version_hint("serde", serde_value)
                        .unwrap()
                        .offset,
                );

                let requests =
                    collect_local_cargo_toml_requests(document_tree, visible_span, true, false);

                assert_eq!(requests.path_dependencies.len(), 1);
                assert!(requests.path_dependencies.contains("../serde"));
                assert!(!requests.path_dependencies.contains("../tokio"));
                assert!(requests.workspace_dependencies.is_empty());
            },
        );
    }

    #[test]
    fn collects_local_cargo_toml_requests_for_visible_workspace_default_feature_hints() {
        with_document_tree(
            r#"
            [dependencies]
            serde = { workspace = true, features = ["derive"] }
            tokio = { workspace = true, features = ["rt"] }
            "#,
            |document_tree| {
                let (_, serde_value) =
                    dig_keys(document_tree, &["dependencies", "serde"]).expect("expected serde");
                let visible_span = tombi_text::Span::empty(
                    local_default_features_request_offset(serde_value)
                        .expect("expected feature offset"),
                );

                let requests =
                    collect_local_cargo_toml_requests(document_tree, visible_span, false, true);

                assert!(requests.path_dependencies.is_empty());
                assert_eq!(requests.workspace_dependencies.len(), 1);
                assert!(requests.workspace_dependencies.contains("serde"));
                assert!(!requests.workspace_dependencies.contains("tokio"));
            },
        );
    }

    #[test]
    fn skips_local_cargo_toml_requests_when_local_hints_are_offscreen() {
        with_document_tree(
            r#"
            [dependencies]
            serde = { path = "../serde" }
            "#,
            |document_tree| {
                let requests = collect_local_cargo_toml_requests(
                    document_tree,
                    tombi_text::Span::empty(tombi_text::Offset::new(0)),
                    true,
                    true,
                );

                assert!(requests.path_dependencies.is_empty());
                assert!(requests.workspace_dependencies.is_empty());
            },
        );
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
        with_value(r#"value = ["tombi", "cargo"]"#, |value| {
            assert_eq!(
                workspace_value_hint_label(value),
                Some(r#" → ["tombi", "cargo"]"#.to_string())
            );
        });
    }

    #[test]
    fn normalizes_workspace_value_hint_label_to_single_line() {
        with_value(
            r#"
            value = """
            hello
            world
            """
            "#,
            |value| {
                let label = workspace_value_hint_label(value).expect("expected label");
                assert!(!label.contains('\n'));
                assert!(label.contains("\\n"));
            },
        );
    }

    #[test]
    fn truncates_workspace_value_hint_label() {
        with_value(
            r#"value = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz""#,
            |value| {
                let label = workspace_value_hint_label(value).expect("expected label");
                assert!(label.ends_with("..."));
                assert!(label.chars().count() <= MAX_WORKSPACE_VALUE_HINT_CHARS + 3);
            },
        );
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

        let uri = tombi_uri::Uri::from_file_path(&cargo_toml_path).expect("expected uri");
        let features = tombi_config::CargoExtensionFeatures::Features(
            tombi_config::CargoExtensionFeatureTree {
                lsp: Some(tombi_config::CargoLspFeatures::Features(
                    tombi_config::CargoLspFeatureTree {
                        inlay_hint: Some(tombi_config::CargoInlayHintFeatures::Features(
                            tombi_config::CargoInlayHintFeatureTree {
                                dependency_version: Some(tombi_config::ToggleFeatureDefaultTrue {
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

        crate::cargo_toml::load_cargo_toml(
            &cargo_toml_path,
            TomlVersion::default(),
            |document_tree, line_index| {
                let hints = inlay_hint_impl(
                    &uri,
                    document_tree,
                    line_index.span(
                        tombi_text::Range::new(
                            tombi_text::Position::new(0, 0),
                            tombi_text::Position::new(8, 0),
                        ),
                        EncodingKind::GraphemeCluster,
                    ),
                    None,
                    LocalCargoTomlCache::default(),
                    TomlVersion::default(),
                    Some(&features),
                )
                .expect("expected inlay hint result");

                assert_eq!(
                    hints,
                    Some(vec![InlayHint {
                        offset: line_index.offset(
                            tombi_text::Position::new(3, 40),
                            EncodingKind::GraphemeCluster,
                        ),
                        label: r#" → "0.0.0-dev""#.to_string(),
                        kind: Some(InlayHintKind::TYPE),
                        tooltip: Some(WORKSPACE_INHERITED_VALUE_TOOLTIP.to_string()),
                        padding_left: Some(true),
                        padding_right: Some(false),
                    }])
                );
            },
        )
        .expect("expected Cargo.toml");
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

        let uri = tombi_uri::Uri::from_file_path(&cargo_toml_path).expect("expected uri");
        let features = tombi_config::CargoExtensionFeatures::Features(
            tombi_config::CargoExtensionFeatureTree {
                lsp: Some(tombi_config::CargoLspFeatures::Features(
                    tombi_config::CargoLspFeatureTree {
                        inlay_hint: Some(tombi_config::CargoInlayHintFeatures::Features(
                            tombi_config::CargoInlayHintFeatureTree {
                                dependency_version: None,
                                default_features: None,
                                workspace_value: Some(tombi_config::ToggleFeatureDefaultTrue {
                                    enabled: Some(false.into()),
                                }),
                            },
                        )),
                        ..Default::default()
                    },
                )),
            },
        );

        crate::cargo_toml::load_cargo_toml(
            &cargo_toml_path,
            TomlVersion::default(),
            |document_tree, line_index| {
                let hints = inlay_hint_impl(
                    &uri,
                    document_tree,
                    line_index.span(
                        tombi_text::Range::new(
                            tombi_text::Position::new(0, 0),
                            tombi_text::Position::new(8, 0),
                        ),
                        EncodingKind::GraphemeCluster,
                    ),
                    None,
                    LocalCargoTomlCache::default(),
                    TomlVersion::default(),
                    Some(&features),
                )
                .expect("expected inlay hint result");

                assert_eq!(hints, None);
            },
        )
        .expect("expected Cargo.toml");
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
            Some("1.0.228")
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
            Some("1.47.1")
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
