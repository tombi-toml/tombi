use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;
use tombi_schema_store::matches_accessors;

use crate::{
    PyprojectNavigationFeature, classify_pyproject_navigation_feature,
    goto_definition_for_member_pyproject_toml, goto_definition_for_workspace_pyproject_toml,
};

pub async fn goto_declaration(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::Location>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is pyproject.toml
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(Default::default());
    }
    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        return Ok(Default::default());
    };

    if !pyproject_goto_declaration_enabled(features, accessors) {
        return Ok(None);
    }

    let locations = if matches_accessors!(accessors, ["tool", "uv", "sources", _, "workspace"]) {
        goto_definition_for_member_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
            false,
        )?
    } else if matches_accessors!(
        accessors[..accessors.len().min(3)],
        ["tool", "uv", "workspace"]
    ) {
        goto_definition_for_workspace_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )?
    } else {
        Vec::with_capacity(0)
    };

    if locations.is_empty() {
        return Ok(Default::default());
    }

    Ok(Some(locations))
}

fn pyproject_goto_declaration_enabled(
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
    accessors: &[tombi_schema_store::Accessor],
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.goto_declaration())
        .and_then(
            |goto_declaration| match classify_pyproject_navigation_feature(accessors) {
                PyprojectNavigationFeature::Dependency => goto_declaration.dependency(),
                PyprojectNavigationFeature::Member => goto_declaration.member(),
                PyprojectNavigationFeature::Path => None,
            },
        )
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
}

pub fn get_current_declaration(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_uri: &tombi_uri::Uri,
) -> Option<tombi_extension::Location> {
    if !pyproject_toml_uri.path().ends_with("pyproject.toml") {
        return None;
    }

    if !matches_accessors!(accessors, ["dependency-groups", _]) {
        return None;
    }

    let (group_key, _) = dig_keys(
        document_tree,
        &["dependency-groups", accessors.get(1)?.as_key()?],
    )?;

    Some(tombi_extension::Location {
        uri: pyproject_toml_uri.clone(),
        range: group_key.unquoted_range(),
    })
}
