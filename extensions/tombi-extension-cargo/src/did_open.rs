use std::{collections::BTreeSet, path::Path};

use futures::stream::{self, StreamExt};
use tombi_config::{CargoExtensionFeatures, TomlVersion};
use tombi_document_tree::{DocumentTree, Table, Value, dig_keys};
use tombi_extension::remote_cache::warm_remote_json_cache;

use crate::{
    cargo_lock::{exact_crates_io_version, load_cached_cargo_lock},
    find_workspace_cargo_toml, get_workspace_path, sanitize_dependency_key,
};

const PREFETCH_CONCURRENCY: usize = 10;

#[derive(Default)]
struct PrefetchUrls {
    awaited: BTreeSet<String>,
    background: BTreeSet<String>,
}

impl PrefetchUrls {
    fn is_empty(&self) -> bool {
        self.awaited.is_empty() && self.background.is_empty()
    }
}

#[derive(Debug, Clone)]
struct RegistryDependency {
    crate_name: String,
    version_requirement: Option<String>,
    default_features_hint: bool,
}

pub async fn did_open(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &DocumentTree,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&CargoExtensionFeatures>,
) -> Result<Option<tokio::task::JoinHandle<bool>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }

    if !cargo_did_open_enabled(features) {
        return Ok(None);
    }

    if warming_disabled(offline, cache_options) {
        return Ok(None);
    }

    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    let document_tree = document_tree.clone();
    let cache_options = cache_options.cloned();
    let features = features.cloned();
    let handle = tokio::spawn(async move {
        let urls = collect_prefetch_urls(
            &document_tree,
            &cargo_toml_path,
            toml_version,
            features.as_ref(),
        )
        .await;
        if urls.is_empty() {
            return false;
        }

        if !urls.background.is_empty() {
            let background_urls = urls.background;
            let background_cache_options = cache_options.clone();
            tokio::spawn(async move {
                warm_urls(background_urls, offline, background_cache_options).await;
            });
        }

        if urls.awaited.is_empty() {
            return false;
        }

        warm_urls(urls.awaited, offline, cache_options).await;
        true
    });

    Ok(Some(handle))
}

async fn warm_urls(
    urls: BTreeSet<String>,
    offline: bool,
    cache_options: Option<tombi_cache::Options>,
) {
    let cache_options = cache_options.as_ref();
    stream::iter(urls)
        .for_each_concurrent(Some(PREFETCH_CONCURRENCY), |url| async move {
            let _ = warm_remote_json_cache(&url, offline, cache_options).await;
        })
        .await;
}

fn cargo_did_open_enabled(features: Option<&CargoExtensionFeatures>) -> bool {
    features.map_or(true, CargoExtensionFeatures::enabled)
}

fn warming_disabled(offline: bool, cache_options: Option<&tombi_cache::Options>) -> bool {
    offline
        || cache_options
            .and_then(|options| options.no_cache)
            .unwrap_or_default()
}

async fn collect_prefetch_urls(
    document_tree: &DocumentTree,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
    features: Option<&CargoExtensionFeatures>,
) -> PrefetchUrls {
    let warm_hover = features.map_or(
        true,
        CargoExtensionFeatures::dependency_detail_hover_enabled,
    );
    let warm_versions = features.map_or(
        true,
        CargoExtensionFeatures::dependency_version_completion_enabled,
    );
    let warm_feature_details = features.map_or(true, |features| {
        features.dependency_feature_completion_enabled()
            || features.default_features_inlay_hint_enabled()
    });
    let prioritize_inlay_hint = features.map_or(
        true,
        CargoExtensionFeatures::default_features_inlay_hint_enabled,
    );

    if !warm_hover && !warm_versions && !warm_feature_details && !prioritize_inlay_hint {
        return PrefetchUrls::default();
    }

    // Resolve workspace document tree once to avoid re-reading/parsing per dependency.
    let workspace = find_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_path(document_tree),
        toml_version,
    );
    let workspace_document_tree = workspace.as_ref().map(|(_, _, dt)| dt);
    let registry_dependencies =
        collect_registry_dependencies(document_tree, workspace_document_tree);
    let cargo_lock = if warm_feature_details || prioritize_inlay_hint {
        load_cached_cargo_lock(cargo_toml_path, toml_version).await
    } else {
        None
    };
    let mut urls = PrefetchUrls::default();

    for dependency in registry_dependencies {
        if warm_hover {
            urls.background.insert(format!(
                "https://crates.io/api/v1/crates/{}",
                dependency.crate_name
            ));
        }

        if warm_versions {
            urls.background.insert(format!(
                "https://crates.io/api/v1/crates/{}/versions",
                dependency.crate_name
            ));
        }

        let resolved_version =
            dependency
                .version_requirement
                .as_deref()
                .and_then(|version_requirement| {
                    cargo_lock
                        .as_ref()
                        .and_then(|lock| {
                            lock.resolve_dependency_version(
                                &dependency.crate_name,
                                version_requirement,
                            )
                        })
                        .or_else(|| exact_crates_io_version(version_requirement))
                });

        if prioritize_inlay_hint && dependency.default_features_hint {
            if let Some(resolved_version) = resolved_version.as_deref() {
                urls.awaited.insert(format!(
                    "https://crates.io/api/v1/crates/{}/{}",
                    dependency.crate_name, resolved_version
                ));
            }
        }

        if warm_feature_details {
            if let Some(resolved_version) = resolved_version {
                urls.background.insert(format!(
                    "https://crates.io/api/v1/crates/{}/{}",
                    dependency.crate_name, resolved_version
                ));
            } else {
                urls.background.insert(format!(
                    "https://crates.io/api/v1/crates/{}",
                    dependency.crate_name
                ));
            }
        }
    }

    for awaited_url in &urls.awaited {
        urls.background.remove(awaited_url);
    }

    urls
}

fn collect_registry_dependencies(
    document_tree: &DocumentTree,
    workspace_document_tree: Option<&DocumentTree>,
) -> Vec<RegistryDependency> {
    let mut dependencies = Vec::new();

    if let Some((_, Value::Table(workspace_dependencies))) =
        dig_keys(document_tree, &["workspace", "dependencies"])
    {
        collect_registry_dependencies_from_table(
            workspace_dependencies,
            "dependencies",
            workspace_document_tree,
            &mut dependencies,
        );
    }

    for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some((_, Value::Table(dependency_table))) =
            dig_keys(document_tree, &[dependency_kind])
        {
            collect_registry_dependencies_from_table(
                dependency_table,
                dependency_kind,
                workspace_document_tree,
                &mut dependencies,
            );
        }
    }

    if let Some((_, Value::Table(targets))) = dig_keys(document_tree, &["target"]) {
        for target in targets.values() {
            let Value::Table(target) = target else {
                continue;
            };

            for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some((_, Value::Table(dependency_table))) =
                    dig_keys(target, &[dependency_kind])
                {
                    collect_registry_dependencies_from_table(
                        dependency_table,
                        dependency_kind,
                        workspace_document_tree,
                        &mut dependencies,
                    );
                }
            }
        }
    }

    dependencies
}

fn collect_registry_dependencies_from_table(
    dependencies: &Table,
    dependency_kind: &str,
    workspace_document_tree: Option<&DocumentTree>,
    registry_dependencies: &mut Vec<RegistryDependency>,
) {
    for (dependency_key, dependency_value) in dependencies.key_values() {
        if let Some(dependency) = registry_dependency(
            dependency_key.value.as_str(),
            dependency_value,
            dependency_kind,
            workspace_document_tree,
        ) {
            registry_dependencies.push(dependency);
        }
    }
}

fn registry_dependency(
    dependency_key: &str,
    dependency_value: &Value,
    dependency_kind: &str,
    workspace_document_tree: Option<&DocumentTree>,
) -> Option<RegistryDependency> {
    match dependency_value {
        Value::String(version_requirement) => Some(RegistryDependency {
            crate_name: dependency_key.to_string(),
            version_requirement: Some(version_requirement.value().to_string()),
            default_features_hint: false,
        }),
        Value::Table(table) => {
            if table.contains_key("path")
                || table.contains_key("git")
                || table.contains_key("registry")
            {
                return None;
            }

            if let Some(Value::Boolean(workspace)) = table.get("workspace")
                && workspace.value()
            {
                return workspace_registry_dependency(
                    dependency_key,
                    dependency_kind,
                    workspace_document_tree,
                );
            }

            Some(RegistryDependency {
                crate_name: match table.get("package") {
                    Some(Value::String(package)) => package.value().to_string(),
                    _ => dependency_key.to_string(),
                },
                version_requirement: match table.get("version") {
                    Some(Value::String(version)) => Some(version.value().to_string()),
                    _ => None,
                },
                default_features_hint: table.get("version").is_some()
                    && table.get("features").is_some()
                    && !dependency_table_default_features_disabled(table),
            })
        }
        _ => None,
    }
}

fn workspace_registry_dependency(
    dependency_key: &str,
    dependency_kind: &str,
    workspace_document_tree: Option<&DocumentTree>,
) -> Option<RegistryDependency> {
    let workspace_document_tree = workspace_document_tree?;

    let (_, workspace_dependency_value) = dig_keys(
        workspace_document_tree,
        &[
            "workspace",
            sanitize_dependency_key(dependency_kind),
            dependency_key,
        ],
    )?;

    match workspace_dependency_value {
        Value::String(version_requirement) => Some(RegistryDependency {
            crate_name: dependency_key.to_string(),
            version_requirement: Some(version_requirement.value().to_string()),
            default_features_hint: false,
        }),
        Value::Table(table) => {
            if table.contains_key("path")
                || table.contains_key("git")
                || table.contains_key("registry")
                || table.contains_key("workspace")
            {
                return None;
            }

            Some(RegistryDependency {
                crate_name: match table.get("package") {
                    Some(Value::String(package)) => package.value().to_string(),
                    _ => dependency_key.to_string(),
                },
                version_requirement: match table.get("version") {
                    Some(Value::String(version)) => Some(version.value().to_string()),
                    _ => None,
                },
                default_features_hint: table.get("version").is_some()
                    && table.get("features").is_some()
                    && !dependency_table_default_features_disabled(table),
            })
        }
        _ => None,
    }
}

fn dependency_table_default_features_disabled(table: &Table) -> bool {
    table
        .get("default-features")
        .is_some_and(|value| matches!(value, Value::Boolean(boolean) if !boolean.value()))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tombi_ast::AstNode;
    use tombi_config::{
        BoolDefaultTrue, CargoCompletionFeatureTree, CargoCompletionFeatures,
        CargoExtensionFeatureTree, CargoExtensionFeatures, CargoHoverFeatureTree,
        CargoHoverFeatures, CargoInlayHintFeatureTree, CargoInlayHintFeatures, CargoLspFeatureTree,
        CargoLspFeatures, ToggleFeature,
    };
    use tombi_document_tree::TryIntoDocumentTree;

    use super::*;

    fn parse_document_tree(source: &str) -> DocumentTree {
        let root = tombi_ast::Root::cast(tombi_parser::parse(source).into_syntax_node()).unwrap();
        root.try_into_document_tree(TomlVersion::default()).unwrap()
    }

    fn uri_for(path: &Path) -> tombi_uri::Uri {
        tombi_uri::Uri::from_file_path(path).unwrap()
    }

    fn sorted_urls(urls: &BTreeSet<String>) -> Vec<String> {
        urls.iter().cloned().collect()
    }

    fn disabled_toggle() -> ToggleFeature {
        ToggleFeature {
            enabled: Some(BoolDefaultTrue(false)),
        }
    }

    fn default_features_only() -> CargoExtensionFeatures {
        CargoExtensionFeatures::Features(CargoExtensionFeatureTree {
            lsp: Some(CargoLspFeatures::Features(CargoLspFeatureTree {
                completion: Some(CargoCompletionFeatures::Features(
                    CargoCompletionFeatureTree {
                        dependency_version: Some(disabled_toggle()),
                        dependency_feature: Some(disabled_toggle()),
                        path: Some(disabled_toggle()),
                    },
                )),
                inlay_hint: Some(CargoInlayHintFeatures::Features(
                    CargoInlayHintFeatureTree {
                        dependency_version: Some(disabled_toggle()),
                        default_features: None,
                        workspace_value: Some(disabled_toggle()),
                    },
                )),
                hover: Some(CargoHoverFeatures::Features(CargoHoverFeatureTree {
                    dependency_detail: Some(disabled_toggle()),
                })),
                ..Default::default()
            })),
        })
    }

    #[tokio::test(flavor = "current_thread")]
    async fn collects_registry_dependencies_from_member_and_workspace_sections() {
        let document_tree = parse_document_tree(
            r#"
            [workspace.dependencies]
            serde_toml = { version = "0.1", package = "toml" }

            [dependencies]
            serde = "1"

            [target.'cfg(unix)'.dependencies]
            tokio = { version = "1" }
            "#,
        );

        let urls = collect_prefetch_urls(
            &document_tree,
            Path::new("/tmp/Cargo.toml"),
            TomlVersion::default(),
            None,
        )
        .await;

        assert!(
            urls.background
                .contains("https://crates.io/api/v1/crates/serde")
        );
        assert!(
            urls.background
                .contains("https://crates.io/api/v1/crates/toml")
        );
        assert!(
            urls.background
                .contains("https://crates.io/api/v1/crates/tokio/versions")
        );
        assert!(urls.awaited.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn excludes_path_git_and_registry_dependencies() {
        let document_tree = parse_document_tree(
            r#"
            [dependencies]
            local = { path = "../local" }
            gitdep = { git = "https://example.com/repo.git" }
            alt = { version = "1", registry = "internal" }
            serde = "1"
            "#,
        );

        let urls = collect_prefetch_urls(
            &document_tree,
            Path::new("/tmp/Cargo.toml"),
            TomlVersion::default(),
            None,
        )
        .await;

        assert_eq!(
            sorted_urls(&urls.background),
            vec![
                "https://crates.io/api/v1/crates/serde".to_string(),
                "https://crates.io/api/v1/crates/serde/versions".to_string(),
            ]
        );
        assert!(urls.awaited.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn excludes_workspace_local_dependencies() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_path = temp_dir.path().join("Cargo.toml");
        let member_dir = temp_dir.path().join("member");
        std::fs::create_dir_all(&member_dir).unwrap();
        std::fs::write(
            &workspace_path,
            r#"
            [workspace]
            members = ["member"]

            [workspace.dependencies]
            local = { path = "member" }
            serde = "1"
            "#,
        )
        .unwrap();

        let member_path = member_dir.join("Cargo.toml");
        std::fs::write(
            &member_path,
            r#"
            [package]
            name = "member"
            version = "0.1.0"

            [dependencies]
            local = { workspace = true }
            serde = { workspace = true }
            "#,
        )
        .unwrap();

        let document_tree = parse_document_tree(&std::fs::read_to_string(&member_path).unwrap());
        let urls =
            collect_prefetch_urls(&document_tree, &member_path, TomlVersion::default(), None).await;

        assert_eq!(
            sorted_urls(&urls.background),
            vec![
                "https://crates.io/api/v1/crates/serde".to_string(),
                "https://crates.io/api/v1/crates/serde/versions".to_string(),
            ]
        );
        assert!(urls.awaited.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn prioritizes_default_feature_hint_warming_over_background_warming() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let cargo_lock_path = temp_dir.path().join("Cargo.lock");
        std::fs::write(
            &cargo_toml_path,
            r#"
            [package]
            name = "demo"
            version = "0.1.0"

            [dependencies]
            serde = { version = "1", features = ["derive"] }
            "#,
        )
        .unwrap();
        std::fs::write(
            &cargo_lock_path,
            r#"
            version = 3

            [[package]]
            name = "demo"
            version = "0.1.0"
            dependencies = ["serde 1.0.228"]

            [[package]]
            name = "serde"
            version = "1.0.228"
            "#,
        )
        .unwrap();

        let document_tree =
            parse_document_tree(&std::fs::read_to_string(&cargo_toml_path).unwrap());
        let urls = collect_prefetch_urls(
            &document_tree,
            &cargo_toml_path,
            TomlVersion::default(),
            Some(&default_features_only()),
        )
        .await;

        assert_eq!(
            sorted_urls(&urls.awaited),
            vec!["https://crates.io/api/v1/crates/serde/1.0.228".to_string()]
        );
        assert!(urls.background.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn did_open_ignores_non_cargo_documents() {
        let document_tree = parse_document_tree("");
        let uri = tombi_uri::Uri::from_str("file:///tmp/pyproject.toml").unwrap();

        let result = did_open(
            &uri,
            &document_tree,
            TomlVersion::default(),
            true,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
    }

    #[test]
    fn uri_helper_builds_file_uri() {
        let path = Path::new("/tmp/Cargo.toml");
        let uri = uri_for(path);

        assert_eq!(uri.path(), "/tmp/Cargo.toml");
    }
}
