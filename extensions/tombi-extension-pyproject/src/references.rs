use tombi_config::TomlVersion;
use tombi_document_tree::{Value, dig_accessors, dig_keys};
use tombi_schema_store::{Accessor, matches_accessors};

use crate::{
    collect_workspace_project_dependency_definitions, extract_exclude_patterns,
    extract_member_patterns, find_pyproject_toml_paths, find_workspace_pyproject_toml,
    get_workspace_member_dependency_definitions, load_pyproject_toml_document_tree,
    parse_requirement,
};

pub async fn references(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }
    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    if !pyproject_references_enabled(features) {
        return Ok(None);
    }

    if matches_accessors!(accessors, ["project", "name"]) {
        let locations = project_name_reference_locations(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )?;
        return Ok((!locations.is_empty()).then_some(locations));
    } else if matches_accessors!(accessors, ["dependency-groups", _]) {
        let locations =
            crate::include_group_locations(document_tree, accessors, &pyproject_toml_path)?;
        return Ok((!locations.is_empty()).then_some(locations));
    }

    if matches_accessors!(accessors, ["project", "dependencies", _])
        || matches_accessors!(accessors, ["project", "optional-dependencies", _, _])
        || matches_accessors!(accessors, ["dependency-groups", _, _])
    {
        let Some((_, Value::String(dep_str))) = dig_accessors(document_tree, accessors) else {
            return Ok(None);
        };
        let Some(requirement) = parse_requirement(dep_str.value()) else {
            return Ok(None);
        };
        let package_name = requirement.name.as_ref();

        let mut locations = Vec::new();
        if requirement.version_or_url.is_none() {
            locations.extend(collect_workspace_project_dependency_definitions(
                package_name,
                &pyproject_toml_path,
                toml_version,
            ));
        }

        if tombi_document_tree::dig_keys(document_tree, &["tool", "uv", "workspace"]).is_some() {
            locations.extend(get_workspace_member_dependency_definitions(
                document_tree,
                &pyproject_toml_path,
                package_name,
                toml_version,
            ));
        }

        return Ok((!locations.is_empty()).then_some(locations));
    }

    Ok(None)
}

pub(crate) fn project_name_reference_locations(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    debug_assert!(matches_accessors!(accessors, ["project", "name"]));

    let Some((_, Value::String(project_name))) = dig_accessors(document_tree, accessors) else {
        return Ok(Vec::new());
    };

    let mut locations = Vec::new();

    let (workspace_pyproject_toml_path, workspace_document_tree) =
        if dig_keys(document_tree, &["tool", "uv", "workspace"]).is_some() {
            (pyproject_toml_path.to_path_buf(), document_tree.clone())
        } else {
            let Some((workspace_path, _, workspace_document_tree)) =
                find_workspace_pyproject_toml(pyproject_toml_path, toml_version)
            else {
                return Ok(Vec::new());
            };
            (workspace_path, workspace_document_tree)
        };

    collect_project_name_references_in_manifest(
        &mut locations,
        &workspace_document_tree,
        &workspace_pyproject_toml_path,
        project_name.value(),
    );

    let member_patterns = extract_member_patterns(&workspace_document_tree, &[]);
    if member_patterns.is_empty() {
        return Ok(locations);
    }

    let exclude_patterns = extract_exclude_patterns(&workspace_document_tree);
    let Some(workspace_dir_path) = workspace_pyproject_toml_path.parent() else {
        return Ok(locations);
    };

    for (_, member_pyproject_toml_path) in
        find_pyproject_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some(member_document_tree) =
            load_pyproject_toml_document_tree(&member_pyproject_toml_path, toml_version)
        else {
            continue;
        };

        collect_project_name_references_in_manifest(
            &mut locations,
            &member_document_tree,
            &member_pyproject_toml_path,
            project_name.value(),
        );
    }

    Ok(locations)
}

fn collect_project_name_references_in_manifest(
    locations: &mut Vec<tombi_extension::Location>,
    document_tree: &tombi_document_tree::DocumentTree,
    pyproject_toml_path: &std::path::Path,
    project_name: &str,
) {
    let Ok(uri) = tombi_uri::Uri::from_file_path(pyproject_toml_path) else {
        return;
    };

    if let Some((source_key, _)) = dig_keys(document_tree, &["tool", "uv", "sources", project_name])
    {
        locations.push(tombi_extension::Location {
            uri: uri.clone(),
            range: source_key.unquoted_range(),
        });
    }

    for requirement in crate::collect_dependency_requirements_from_document_tree(document_tree) {
        if requirement.requirement.name.as_ref() == project_name {
            locations.push(tombi_extension::Location {
                uri: uri.clone(),
                range: requirement.dependency.unquoted_range(),
            });
        }
    }
}

fn pyproject_references_enabled(
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.references())
        .and_then(|references| references.dependency())
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}
