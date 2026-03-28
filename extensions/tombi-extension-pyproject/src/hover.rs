use std::path::Path;

use pep508_rs::VersionOrUrl;
use serde::Deserialize;
use tombi_config::TomlVersion;
use tombi_document_tree::{Value, dig_accessors, dig_keys};
use tombi_extension::{HoverMetadata, fetch_cached_remote_json};
use tombi_schema_store::{Accessor, matches_accessors};

use crate::{
    find_member_project_toml, find_workspace_pyproject_toml, get_project_name,
    load_pyproject_toml_document_tree, parse_requirement, resolve_member_pyproject_toml_path,
};

#[derive(Debug, Deserialize)]
struct PypiProjectResponse {
    info: PypiProjectInfo,
}

#[derive(Debug, Deserialize)]
struct PypiProjectInfo {
    name: Option<String>,
    summary: Option<String>,
}

pub async fn hover(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<HoverMetadata>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    if matches_accessors!(accessors, ["tool", "uv", "sources", _])
        || matches_accessors!(accessors, ["tool", "uv", "sources", _, _])
    {
        return Ok(resolve_pyproject_source_metadata(
            document_tree,
            &accessors[..4],
            &pyproject_toml_path,
            toml_version,
        ));
    }

    let Some(dependency_accessors) = get_dependency_accessors(accessors) else {
        return Ok(None);
    };

    let Some((_, Value::String(dependency))) = dig_accessors(document_tree, dependency_accessors)
    else {
        return Ok(None);
    };

    let Some(requirement) = parse_requirement(dependency.value()) else {
        return Ok(None);
    };

    let package_name = requirement.name.as_ref();

    if let Some(metadata) = resolve_pyproject_dependency_metadata_from_sources(
        document_tree,
        package_name,
        document_tree,
        &pyproject_toml_path,
        toml_version,
    ) {
        return Ok(Some(metadata));
    }

    if matches!(
        requirement.version_or_url.as_ref(),
        Some(VersionOrUrl::Url(_))
    ) {
        return Ok(None);
    }

    fetch_pypi_metadata(package_name, offline, cache_options).await
}

fn get_dependency_accessors(accessors: &[Accessor]) -> Option<&[Accessor]> {
    if matches_accessors!(accessors, ["project", "dependencies", _]) {
        Some(&accessors[..3])
    } else if matches_accessors!(accessors, ["project", "optional-dependencies", _, _]) {
        Some(&accessors[..4])
    } else if matches_accessors!(accessors, ["dependency-groups", _, _]) {
        Some(&accessors[..3])
    } else {
        None
    }
}

fn resolve_pyproject_dependency_metadata_from_sources(
    document_tree: &tombi_document_tree::DocumentTree,
    package_name: &str,
    current_document_tree: &tombi_document_tree::DocumentTree,
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<HoverMetadata> {
    let Some((_, Value::Table(sources))) = dig_keys(document_tree, &["tool", "uv", "sources"])
    else {
        return None;
    };
    let (package_key, source) = sources.get_key_value(package_name)?;

    resolve_source_value_metadata(
        package_key.value.as_str(),
        source,
        current_document_tree,
        pyproject_toml_path,
        toml_version,
    )
}

fn resolve_pyproject_source_metadata(
    document_tree: &tombi_document_tree::DocumentTree,
    source_accessors: &[Accessor],
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<HoverMetadata> {
    let Some(Accessor::Key(package_name)) = source_accessors.get(3) else {
        return None;
    };
    let (_, source) = dig_accessors(document_tree, source_accessors)?;

    resolve_source_value_metadata(
        package_name,
        source,
        document_tree,
        pyproject_toml_path,
        toml_version,
    )
}

fn resolve_source_value_metadata(
    package_name: &str,
    source: &Value,
    current_document_tree: &tombi_document_tree::DocumentTree,
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<HoverMetadata> {
    let Value::Table(source_table) = source else {
        return None;
    };

    if let Some((_, Value::Boolean(workspace))) = source_table.get_key_value("workspace")
        && workspace.value()
    {
        return resolve_workspace_member_metadata(
            package_name,
            current_document_tree,
            pyproject_toml_path,
            toml_version,
        );
    }

    if let Some((_, Value::String(path))) = source_table.get_key_value("path") {
        let member_pyproject_toml_path =
            resolve_member_pyproject_toml_path(pyproject_toml_path, path.value())?;
        return load_project_metadata(&member_pyproject_toml_path, toml_version);
    }

    None
}

fn resolve_workspace_member_metadata(
    package_name: &str,
    current_document_tree: &tombi_document_tree::DocumentTree,
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<HoverMetadata> {
    let member_pyproject_toml_path =
        if dig_keys(current_document_tree, &["tool", "uv", "workspace"]).is_some() {
            let (member_pyproject_toml_path, _) = find_member_project_toml(
                package_name,
                current_document_tree,
                pyproject_toml_path,
                toml_version,
            )?;
            member_pyproject_toml_path
        } else {
            let (workspace_pyproject_toml_path, _, workspace_document_tree) =
                find_workspace_pyproject_toml(pyproject_toml_path, toml_version)?;
            let (member_pyproject_toml_path, _) = find_member_project_toml(
                package_name,
                &workspace_document_tree,
                &workspace_pyproject_toml_path,
                toml_version,
            )?;
            member_pyproject_toml_path
        };

    load_project_metadata(&member_pyproject_toml_path, toml_version)
}

fn load_project_metadata(
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<HoverMetadata> {
    let document_tree = load_pyproject_toml_document_tree(pyproject_toml_path, toml_version)?;
    let project_name = get_project_name(&document_tree).map(|name| name.value().to_string());
    let description = match dig_keys(&document_tree, &["project", "description"]) {
        Some((_, Value::String(description))) => Some(description.value().to_string()),
        _ => None,
    };

    if project_name.is_none() && description.is_none() {
        return None;
    }

    Some(HoverMetadata {
        title: project_name,
        description,
    })
}

async fn fetch_pypi_metadata(
    package_name: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<HoverMetadata>, tower_lsp::jsonrpc::Error> {
    let url = format!("https://pypi.org/pypi/{package_name}/json");
    let Some(response) =
        fetch_cached_remote_json::<PypiProjectResponse>(&url, offline, cache_options).await
    else {
        return Ok(None);
    };

    if response.info.name.is_none() && response.info.summary.is_none() {
        return Ok(None);
    }

    Ok(Some(HoverMetadata {
        title: response.info.name,
        description: response.info.summary,
    }))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use pep508_rs::{Requirement, VerbatimUrl};

    #[test]
    fn parses_pypi_metadata_response() {
        let response: PypiProjectResponse = serde_json::from_str(
            r#"{
                "info": {
                    "name": "requests",
                    "summary": "Python HTTP for Humans."
                }
            }"#,
        )
        .unwrap();

        assert_eq!(response.info.name.as_deref(), Some("requests"));
        assert_eq!(
            response.info.summary.as_deref(),
            Some("Python HTTP for Humans.")
        );
    }

    #[test]
    fn rejects_direct_url_requirements_for_remote_lookup() {
        let requirement =
            Requirement::<VerbatimUrl>::from_str("demo @ https://example.com/demo-0.1.0.tar.gz")
                .unwrap();

        assert!(matches!(
            requirement.version_or_url,
            Some(VersionOrUrl::Url(_))
        ));
    }
}
