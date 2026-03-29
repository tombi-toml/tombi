use std::{collections::BTreeSet, path::Path};

use futures::stream::{self, StreamExt};
use tombi_config::{CargoExtensionFeatures, TomlVersion};
use tombi_document_tree::{DocumentTree, Table, Value, dig_keys};
use tombi_extension::warm_remote_json_cache;

use crate::{find_workspace_cargo_toml, get_workspace_path, sanitize_dependency_key};

const PREFETCH_CONCURRENCY: usize = 10;

pub async fn did_open(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &DocumentTree,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&CargoExtensionFeatures>,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(());
    }

    if !cargo_did_open_enabled(features) {
        return Ok(());
    }

    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(());
    };

    let urls = collect_prefetch_urls(document_tree, &cargo_toml_path, toml_version);
    if urls.is_empty() {
        return Ok(());
    }

    let cache_options = cache_options.cloned();
    tokio::spawn(async move {
        stream::iter(urls)
            .for_each_concurrent(Some(PREFETCH_CONCURRENCY), |url| {
                let cache_options = cache_options.clone();
                async move {
                    let _ = warm_remote_json_cache(&url, offline, cache_options.as_ref()).await;
                }
            })
            .await;
    });

    Ok(())
}

fn cargo_did_open_enabled(features: Option<&CargoExtensionFeatures>) -> bool {
    features.map_or(true, CargoExtensionFeatures::enabled)
}

fn collect_prefetch_urls(
    document_tree: &DocumentTree,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Vec<String> {
    let mut crate_names = BTreeSet::new();

    // Resolve workspace document tree once to avoid re-reading/parsing per dependency.
    let workspace = find_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_path(document_tree),
        toml_version,
    );

    if let Some((_, Value::Table(workspace_dependencies))) =
        dig_keys(document_tree, &["workspace", "dependencies"])
    {
        collect_registry_dependency_names(
            workspace_dependencies,
            "dependencies",
            workspace.as_ref().map(|(_, _, dt)| dt),
            &mut crate_names,
        );
    }

    collect_member_registry_dependency_names(
        document_tree,
        workspace.as_ref().map(|(_, _, dt)| dt),
        &mut crate_names,
    );

    crate_names
        .into_iter()
        .flat_map(|crate_name| {
            [
                format!("https://crates.io/api/v1/crates/{crate_name}"),
                format!("https://crates.io/api/v1/crates/{crate_name}/versions"),
            ]
        })
        .collect()
}

fn collect_member_registry_dependency_names(
    document_tree: &DocumentTree,
    workspace_document_tree: Option<&DocumentTree>,
    crate_names: &mut BTreeSet<String>,
) {
    for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some((_, Value::Table(dependencies))) = dig_keys(document_tree, &[dependency_kind]) {
            collect_registry_dependency_names(
                dependencies,
                dependency_kind,
                workspace_document_tree,
                crate_names,
            );
        }
    }

    if let Some((_, Value::Table(targets))) = dig_keys(document_tree, &["target"]) {
        for target in targets.values() {
            let Value::Table(target) = target else {
                continue;
            };

            for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some((_, Value::Table(dependencies))) = dig_keys(target, &[dependency_kind])
                {
                    collect_registry_dependency_names(
                        dependencies,
                        dependency_kind,
                        workspace_document_tree,
                        crate_names,
                    );
                }
            }
        }
    }
}

fn collect_registry_dependency_names(
    dependencies: &Table,
    dependency_kind: &str,
    workspace_document_tree: Option<&DocumentTree>,
    crate_names: &mut BTreeSet<String>,
) {
    for (dependency_key, dependency_value) in dependencies.key_values() {
        if let Some(crate_name) = registry_dependency_name(
            dependency_key.value.as_str(),
            dependency_value,
            dependency_kind,
            workspace_document_tree,
        ) {
            crate_names.insert(crate_name);
        }
    }
}

fn registry_dependency_name(
    dependency_key: &str,
    dependency_value: &Value,
    dependency_kind: &str,
    workspace_document_tree: Option<&DocumentTree>,
) -> Option<String> {
    match dependency_value {
        Value::String(_) => Some(dependency_key.to_string()),
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
                return workspace_registry_dependency_name(
                    dependency_key,
                    dependency_kind,
                    workspace_document_tree,
                );
            }

            Some(match table.get("package") {
                Some(Value::String(package)) => package.value().to_string(),
                _ => dependency_key.to_string(),
            })
        }
        _ => None,
    }
}

fn workspace_registry_dependency_name(
    dependency_key: &str,
    dependency_kind: &str,
    workspace_document_tree: Option<&DocumentTree>,
) -> Option<String> {
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
        Value::String(_) => Some(dependency_key.to_string()),
        Value::Table(table) => {
            if table.contains_key("path")
                || table.contains_key("git")
                || table.contains_key("registry")
                || table.contains_key("workspace")
            {
                return None;
            }

            Some(match table.get("package") {
                Some(Value::String(package)) => package.value().to_string(),
                _ => dependency_key.to_string(),
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tombi_ast::AstNode;
    use tombi_document_tree::TryIntoDocumentTree;

    use super::*;

    fn parse_document_tree(source: &str) -> DocumentTree {
        let root = tombi_ast::Root::cast(tombi_parser::parse(source).into_syntax_node()).unwrap();
        root.try_into_document_tree(TomlVersion::default()).unwrap()
    }

    fn uri_for(path: &Path) -> tombi_uri::Uri {
        tombi_uri::Uri::from_file_path(path).unwrap()
    }

    #[test]
    fn collects_registry_dependencies_from_member_and_workspace_sections() {
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
        );

        assert!(urls.contains(&"https://crates.io/api/v1/crates/serde".to_string()));
        assert!(urls.contains(&"https://crates.io/api/v1/crates/toml".to_string()));
        assert!(urls.contains(&"https://crates.io/api/v1/crates/tokio/versions".to_string()));
    }

    #[test]
    fn excludes_path_git_and_registry_dependencies() {
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
        );

        assert_eq!(
            urls,
            vec![
                "https://crates.io/api/v1/crates/serde".to_string(),
                "https://crates.io/api/v1/crates/serde/versions".to_string(),
            ]
        );
    }

    #[test]
    fn excludes_workspace_local_dependencies() {
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
        let urls = collect_prefetch_urls(&document_tree, &member_path, TomlVersion::default());

        assert_eq!(
            urls,
            vec![
                "https://crates.io/api/v1/crates/serde".to_string(),
                "https://crates.io/api/v1/crates/serde/versions".to_string(),
            ]
        );
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
