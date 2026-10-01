use std::{collections::BTreeSet, path::Path};

use futures::stream::{self, StreamExt};
use tombi_config::{PyprojectExtensionFeatures, TomlVersion};
use tombi_document_tree_syntax::{DocumentTree, Table, Value, dig_keys};
use tombi_extension::remote_cache::warm_remote_json_cache;
use tombi_future::Boxable;

use crate::{
    UNUSED_ENCODING, collect_all_dependency_requirements_from_document_tree,
    find_workspace_pyproject_toml,
};

const PREFETCH_CONCURRENCY: usize = 10;

pub fn did_open(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &DocumentTree<'_>,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&PyprojectExtensionFeatures>,
) -> Option<tombi_future::BoxFuture<'static, ()>> {
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return None;
    }

    if !features
        .map(|features| features.enabled())
        .unwrap_or_default()
        .value()
    {
        return None;
    }

    if !features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.hover())
        .and_then(|hover| hover.dependency_detail())
        .map(|dependency_detail| dependency_detail.enabled())
        .unwrap_or_default()
        .value()
    {
        return None;
    }

    if warming_disabled(offline, cache_options) {
        return None;
    }

    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        return None;
    };

    // The tree borrows the document, so the URLs are collected before the future outlives it.
    let urls = collect_prefetch_urls(document_tree, &pyproject_toml_path, toml_version);
    let cache_options = cache_options.cloned();
    let warm = async move {
        if urls.is_empty() {
            return;
        }

        let cache_options = cache_options.as_ref();
        stream::iter(urls)
            .for_each_concurrent(Some(PREFETCH_CONCURRENCY), |url| async move {
                let _ = warm_remote_json_cache(&url, offline, cache_options).await;
            })
            .await;
    };

    Some(warm.boxed())
}

fn warming_disabled(offline: bool, cache_options: Option<&tombi_cache::Options>) -> bool {
    offline
        || cache_options
            .and_then(|options| options.no_cache)
            .unwrap_or_default()
}

fn collect_prefetch_urls(
    document_tree: &DocumentTree<'_>,
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Vec<String> {
    let current_sources = pyproject_sources(document_tree);
    let mut package_names = BTreeSet::new();

    for dependency_requirement in
        collect_all_dependency_requirements_from_document_tree(document_tree)
    {
        if matches!(
            dependency_requirement.version_or_url(),
            Some(pep508_rs::VersionOrUrl::Url(_))
        ) {
            continue;
        }

        let package_name = dependency_requirement.requirement.name.as_ref();
        if has_source_override(current_sources, package_name) {
            continue;
        }

        package_names.insert(package_name.to_string());
    }

    remove_workspace_source_overrides(
        &mut package_names,
        document_tree,
        pyproject_toml_path,
        toml_version,
    );

    package_names
        .into_iter()
        .map(|package_name| format!("https://pypi.org/pypi/{package_name}/json"))
        .collect()
}

fn pyproject_sources<'a, 't>(document_tree: &'a DocumentTree<'t>) -> Option<&'a Table<'t>> {
    match dig_keys(document_tree, &["tool", "uv", "sources"]) {
        Some((_, Value::Table(sources))) => Some(sources),
        _ => None,
    }
}

fn remove_workspace_source_overrides(
    package_names: &mut BTreeSet<String>,
    document_tree: &DocumentTree<'_>,
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) {
    if package_names.is_empty() || dig_keys(document_tree, &["tool", "uv", "workspace"]).is_some() {
        return;
    }

    find_workspace_pyproject_toml(
        pyproject_toml_path,
        toml_version,
        UNUSED_ENCODING,
        |workspace_pyproject_toml_path, _, workspace_document_tree, _| {
            if workspace_pyproject_toml_path == pyproject_toml_path {
                return;
            }

            if let Some(sources) = pyproject_sources(workspace_document_tree) {
                package_names.retain(|package_name| !sources.contains_key(package_name.as_str()));
            }
        },
    );
}

fn has_source_override(sources: Option<&Table<'_>>, package_name: &str) -> bool {
    sources.is_some_and(|sources| sources.contains_key(package_name))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use tombi_ast_syntax::AstNode as _;
    use tombi_document_tree_syntax::TryIntoDocumentTree;

    use super::*;

    fn with_document_tree(source: &str, f: impl FnOnce(&DocumentTree<'_>)) {
        let parsed = tombi_parser::parse(source);
        let root = parsed.root();
        let decoded = root.decode_strings(TomlVersion::default());
        let document_tree = root
            .try_into_document_tree(TomlVersion::default(), &decoded)
            .unwrap();
        f(&document_tree);
    }

    #[test]
    fn collects_registry_dependencies_without_source_overrides() {
        with_document_tree(
            r#"
            [project]
            dependencies = ["requests>=2.0"]

            [project.optional-dependencies]
            test = ["pytest>=7.0"]

            [dependency-groups]
            dev = ["ruff>=0.3"]
            "#,
            |document_tree| {
                let urls = collect_prefetch_urls(
                    document_tree,
                    Path::new("/tmp/pyproject.toml"),
                    TomlVersion::default(),
                );

                assert_eq!(
                    urls,
                    vec![
                        "https://pypi.org/pypi/pytest/json".to_string(),
                        "https://pypi.org/pypi/requests/json".to_string(),
                        "https://pypi.org/pypi/ruff/json".to_string(),
                    ]
                );
            },
        );
    }

    #[test]
    fn excludes_direct_url_and_source_overrides() {
        with_document_tree(
            r#"
            [project]
            dependencies = [
              "requests>=2.0",
              "demo @ https://example.com/demo-0.1.0.tar.gz",
            ]

            [tool.uv.sources]
            requests = { path = "../requests" }
            "#,
            |document_tree| {
                let urls = collect_prefetch_urls(
                    document_tree,
                    Path::new("/tmp/pyproject.toml"),
                    TomlVersion::default(),
                );

                assert!(urls.is_empty());
            },
        );
    }

    #[test]
    fn excludes_workspace_source_overrides() {
        let temp_dir = tempfile::tempdir().unwrap();
        let workspace_path = temp_dir.path().join("pyproject.toml");
        let member_dir = temp_dir.path().join("member");
        std::fs::create_dir_all(&member_dir).unwrap();
        std::fs::write(
            &workspace_path,
            r#"
            [project]
            name = "workspace"
            version = "0.1.0"

            [tool.uv.workspace]
            members = ["member"]

            [tool.uv.sources]
            requests = { workspace = true }
            "#,
        )
        .unwrap();

        let member_path = member_dir.join("pyproject.toml");
        std::fs::write(
            &member_path,
            r#"
            [project]
            name = "member"
            version = "0.1.0"
            dependencies = ["requests>=2.0", "pytest>=7.0"]
            "#,
        )
        .unwrap();

        with_document_tree(
            &std::fs::read_to_string(&member_path).unwrap(),
            |document_tree| {
                let urls =
                    collect_prefetch_urls(document_tree, &member_path, TomlVersion::default());

                assert_eq!(urls, vec!["https://pypi.org/pypi/pytest/json".to_string()]);
            },
        );
    }

    #[test]
    fn did_open_ignores_non_pyproject_documents() {
        with_document_tree("", |document_tree| {
            let uri = tombi_uri::Uri::from_str("file:///tmp/Cargo.toml").unwrap();

            let result = did_open(
                &uri,
                document_tree,
                TomlVersion::default(),
                true,
                None,
                None,
            );

            assert!(result.is_none());
        });
    }
}
