use tombi_config::TomlVersion;
use tombi_document_tree_syntax::{Value, dig_accessors, dig_keys};
use tombi_extension::SpanConverter;
use tombi_schema_store::{Accessor, matches_accessors};

use crate::{
    extract_exclude_patterns, extract_member_patterns, find_pyproject_toml_paths,
    find_workspace_pyproject_toml, get_workspace_member_dependency_definitions,
    is_dependency_name_accessors, is_project_name_accessors, parse_requirement,
    with_pyproject_toml,
};

pub async fn references(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
    converter: SpanConverter<'_, '_>,
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

    if is_project_name_accessors(accessors) {
        let locations = project_name_reference_locations(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
            converter,
        )?;
        return Ok((!locations.is_empty()).then_some(locations));
    } else if matches_accessors!(accessors, ["dependency-groups", _]) {
        let locations = crate::include_group_locations(
            document_tree,
            accessors,
            &pyproject_toml_path,
            converter,
        )?;
        return Ok((!locations.is_empty()).then_some(locations));
    }

    if is_dependency_name_accessors(accessors) {
        let Some((_, Value::String(dep_str))) = dig_accessors(document_tree, accessors) else {
            return Ok(None);
        };
        let Some(requirement) = parse_requirement(dep_str.value()) else {
            return Ok(None);
        };
        let package_name = requirement.name.as_ref();

        let mut locations = Vec::new();
        if tombi_document_tree_syntax::dig_keys(document_tree, &["tool", "uv", "workspace"])
            .is_some()
        {
            locations.extend(get_workspace_member_dependency_definitions(
                document_tree,
                &pyproject_toml_path,
                package_name,
                toml_version,
                converter.encoding(),
            ));
        }

        return Ok((!locations.is_empty()).then_some(locations));
    }

    Ok(None)
}

pub(crate) fn project_name_reference_locations(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    converter: SpanConverter<'_, '_>,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    debug_assert!(is_project_name_accessors(accessors));

    let Some((_, Value::String(project_name))) = dig_accessors(document_tree, accessors) else {
        return Ok(Vec::new());
    };

    let mut locations = Vec::new();
    let project_name = project_name.value();

    if dig_keys(document_tree, &["tool", "uv", "workspace"]).is_some() {
        collect_workspace_project_name_references(
            &mut locations,
            document_tree,
            pyproject_toml_path,
            project_name,
            toml_version,
            converter,
        );
    } else {
        find_workspace_pyproject_toml(
            pyproject_toml_path,
            toml_version,
            converter.encoding(),
            |workspace_path, _, workspace_document_tree, workspace_converter| {
                collect_workspace_project_name_references(
                    &mut locations,
                    workspace_document_tree,
                    &workspace_path,
                    project_name,
                    toml_version,
                    workspace_converter,
                );
            },
        );
    }

    Ok(locations)
}

fn collect_workspace_project_name_references(
    locations: &mut Vec<tombi_extension::Location>,
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    workspace_pyproject_toml_path: &std::path::Path,
    project_name: &str,
    toml_version: TomlVersion,
    converter: SpanConverter<'_, '_>,
) {
    collect_project_name_references_in_manifest(
        locations,
        workspace_document_tree,
        workspace_pyproject_toml_path,
        project_name,
        converter,
    );

    let member_patterns = extract_member_patterns(workspace_document_tree, &[]);
    if member_patterns.is_empty() {
        return;
    }

    let exclude_patterns = extract_exclude_patterns(workspace_document_tree);
    let Some(workspace_dir_path) = workspace_pyproject_toml_path.parent() else {
        return;
    };

    for (_, member_pyproject_toml_path) in
        find_pyproject_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        with_pyproject_toml(
            &member_pyproject_toml_path,
            toml_version,
            converter.encoding(),
            |_, member_document_tree, member_converter| {
                collect_project_name_references_in_manifest(
                    locations,
                    member_document_tree,
                    &member_pyproject_toml_path,
                    project_name,
                    member_converter,
                );
            },
        );
    }
}

fn collect_project_name_references_in_manifest(
    locations: &mut Vec<tombi_extension::Location>,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    pyproject_toml_path: &std::path::Path,
    project_name: &str,
    converter: SpanConverter<'_, '_>,
) {
    let Ok(uri) = tombi_uri::Uri::from_file_path(pyproject_toml_path) else {
        return;
    };

    if let Some((source_key, _)) = dig_keys(document_tree, &["tool", "uv", "sources", project_name])
    {
        locations.push(converter.location(uri.clone(), source_key.unquoted_span()));
    }

    for requirement in crate::collect_dependency_requirements_from_document_tree(document_tree) {
        if requirement.requirement.name.as_ref() == project_name {
            locations.push(converter.location(uri.clone(), requirement.dependency.unquoted_span()));
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
