use std::{collections::BTreeSet, path::Path};

use futures::stream::{self, StreamExt};
use tombi_config::{PyprojectExtensionFeatures, TomlVersion};
use tombi_document_tree::{DocumentTree, Table, Value, dig_keys};
use tombi_extension::remote_cache::warm_remote_json_cache;

use crate::{
    collect_all_dependency_requirements_from_document_tree, find_workspace_pyproject_toml,
};

const PREFETCH_CONCURRENCY: usize = 10;

pub async fn did_open(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &DocumentTree,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&PyprojectExtensionFeatures>,
) -> Result<(), tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(());
    }

    if !pyproject_did_open_enabled(features) {
        return Ok(());
    }

    if warming_disabled(offline, cache_options) {
        return Ok(());
    }

    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        return Ok(());
    };

    let urls = collect_prefetch_urls(document_tree, &pyproject_toml_path, toml_version);
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

fn pyproject_did_open_enabled(features: Option<&PyprojectExtensionFeatures>) -> bool {
    features.map_or(true, PyprojectExtensionFeatures::enabled)
}

fn warming_disabled(offline: bool, cache_options: Option<&tombi_cache::Options>) -> bool {
    offline
        || cache_options
            .and_then(|options| options.no_cache)
            .unwrap_or_default()
}

fn collect_prefetch_urls(
    document_tree: &DocumentTree,
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Vec<String> {
    let current_sources = pyproject_sources(document_tree);
    let workspace_sources =
        workspace_pyproject_sources(document_tree, pyproject_toml_path, toml_version);
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
        if has_source_override(current_sources, package_name)
            || workspace_sources
                .as_ref()
                .is_some_and(|sources| has_source_override(Some(sources), package_name))
        {
            continue;
        }

        package_names.insert(package_name.to_string());
    }

    package_names
        .into_iter()
        .map(|package_name| format!("https://pypi.org/pypi/{package_name}/json"))
        .collect()
}

fn pyproject_sources(document_tree: &DocumentTree) -> Option<&Table> {
    match dig_keys(document_tree, &["tool", "uv", "sources"]) {
        Some((_, Value::Table(sources))) => Some(sources),
        _ => None,
    }
}

fn workspace_pyproject_sources<'a>(
    document_tree: &'a DocumentTree,
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<Table> {
    if dig_keys(document_tree, &["tool", "uv", "workspace"]).is_some() {
        return None;
    }

    let (workspace_pyproject_toml_path, _, workspace_document_tree) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)?;

    if workspace_pyproject_toml_path == pyproject_toml_path {
        return None;
    }

    pyproject_sources(&workspace_document_tree).cloned()
}

fn has_source_override(sources: Option<&Table>, package_name: &str) -> bool {
    sources.is_some_and(|sources| sources.contains_key(package_name))
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

    #[test]
    fn collects_registry_dependencies_without_source_overrides() {
        let document_tree = parse_document_tree(
            r#"
            [project]
            dependencies = ["requests>=2.0"]

            [project.optional-dependencies]
            test = ["pytest>=7.0"]

            [dependency-groups]
            dev = ["ruff>=0.3"]
            "#,
        );

        let urls = collect_prefetch_urls(
            &document_tree,
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
    }

    #[test]
    fn excludes_direct_url_and_source_overrides() {
        let document_tree = parse_document_tree(
            r#"
            [project]
            dependencies = [
              "requests>=2.0",
              "demo @ https://example.com/demo-0.1.0.tar.gz",
            ]

            [tool.uv.sources]
            requests = { path = "../requests" }
            "#,
        );

        let urls = collect_prefetch_urls(
            &document_tree,
            Path::new("/tmp/pyproject.toml"),
            TomlVersion::default(),
        );

        assert!(urls.is_empty());
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

        let document_tree = parse_document_tree(&std::fs::read_to_string(&member_path).unwrap());
        let urls = collect_prefetch_urls(&document_tree, &member_path, TomlVersion::default());

        assert_eq!(urls, vec!["https://pypi.org/pypi/pytest/json".to_string()]);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn did_open_ignores_non_pyproject_documents() {
        let document_tree = parse_document_tree("");
        let uri = tombi_uri::Uri::from_str("file:///tmp/Cargo.toml").unwrap();

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
}
