use tombi_config::TomlVersion;
use tombi_document_tree::{Value, dig_accessors};
use tombi_schema_store::matches_accessors;

use crate::{
    collect_workspace_project_dependency_definitions, get_workspace_member_dependency_definitions,
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

    if matches_accessors!(accessors, ["dependency-groups", _]) {
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
