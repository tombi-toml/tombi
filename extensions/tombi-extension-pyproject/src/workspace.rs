use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_document_tree_syntax::dig_accessors;
use tombi_schema_store::matches_accessors;

use tombi_extension::SpanConverter;
use tombi_text::EncodingKind;

use crate::{
    PackageLocation, find_workspace_pyproject_toml, get_project_name, manifest::with_pyproject_toml,
};

pub(crate) fn extract_member_patterns<'a, 't>(
    workspace_document_tree: &'a tombi_document_tree_syntax::DocumentTree<'t>,
    accessors: &[tombi_schema_store::Accessor],
) -> Vec<&'a tombi_document_tree_syntax::String<'t>> {
    if matches_accessors!(accessors, ["tool", "uv", "workspace", "members", _]) {
        let Some((_, tombi_document_tree_syntax::Value::String(member))) =
            dig_accessors(workspace_document_tree, accessors)
        else {
            return vec![];
        };
        vec![member]
    } else {
        match tombi_document_tree_syntax::dig_keys(
            workspace_document_tree,
            &["tool", "uv", "workspace", "members"],
        ) {
            Some((_, tombi_document_tree_syntax::Value::Array(members))) => members
                .iter()
                .filter_map(|member| match member {
                    tombi_document_tree_syntax::Value::String(member_pattern) => {
                        Some(member_pattern)
                    }
                    _ => None,
                })
                .collect_vec(),
            _ => vec![],
        }
    }
}

pub(crate) fn extract_exclude_patterns<'a, 't>(
    workspace_document_tree: &'a tombi_document_tree_syntax::DocumentTree<'t>,
) -> Vec<&'a tombi_document_tree_syntax::String<'t>> {
    match tombi_document_tree_syntax::dig_keys(
        workspace_document_tree,
        &["tool", "uv", "workspace", "exclude"],
    ) {
        Some((_, tombi_document_tree_syntax::Value::Array(exclude))) => exclude
            .iter()
            .filter_map(|member| match member {
                tombi_document_tree_syntax::Value::String(member_pattern) => Some(member_pattern),
                _ => None,
            })
            .collect_vec(),
        _ => Vec::new(),
    }
}

pub(crate) fn find_pyproject_toml_paths<'a, 't>(
    member_patterns: &'a [&'a tombi_document_tree_syntax::String<'t>],
    exclude_patterns: &'a [&'a tombi_document_tree_syntax::String<'t>],
    workspace_dir_path: &'a std::path::Path,
) -> impl Iterator<
    Item = (
        &'a tombi_document_tree_syntax::String<'t>,
        std::path::PathBuf,
    ),
> + 'a {
    let exclude_patterns = exclude_patterns
        .iter()
        .filter_map(|pattern| glob::Pattern::new(pattern.value()).ok())
        .collect_vec();

    member_patterns
        .iter()
        .filter_map(move |&member_pattern| {
            let mut manifest_paths = vec![];

            let mut member_pattern_path =
                std::path::Path::new(member_pattern.value()).to_path_buf();
            if !member_pattern_path.is_absolute() {
                member_pattern_path = workspace_dir_path.join(member_pattern_path);
            }

            let candidate_paths = match tombi_fs::glob(&member_pattern_path.to_string_lossy()) {
                Ok(paths) => paths,
                Err(_) => return None,
            };

            for candidate_path in candidate_paths {
                if !tombi_fs::is_dir(&candidate_path) {
                    continue;
                }

                let manifest_path = candidate_path.join("pyproject.toml");
                if !tombi_fs::is_file(&manifest_path) {
                    continue;
                }

                let is_excluded = exclude_patterns.iter().any(|exclude_pattern| {
                    exclude_pattern.matches(&manifest_path.to_string_lossy())
                });

                if !is_excluded {
                    manifest_paths.push((member_pattern, manifest_path));
                }
            }

            (!manifest_paths.is_empty()).then_some(manifest_paths)
        })
        .flatten()
}

pub(crate) fn goto_definition_for_member_pyproject_toml(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    converter: SpanConverter<'_, '_>,
    jump_to_package: bool,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    if matches_accessors!(accessors, ["tool", "uv", "sources", _])
        || matches_accessors!(accessors, ["tool", "uv", "sources", _, "workspace"])
    {
        match goto_workspace_member(
            document_tree,
            accessors,
            pyproject_toml_path,
            toml_version,
            converter,
            jump_to_package,
        )? {
            Some(location) => Ok(vec![location]),
            None => Ok(Vec::new()),
        }
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn goto_definition_for_workspace_pyproject_toml(
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[tombi_schema_store::Accessor],
    workspace_pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    encoding: EncodingKind,
) -> Result<Vec<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    if matches_accessors!(accessors, ["tool", "uv", "workspace", "members"])
        || matches_accessors!(accessors, ["tool", "uv", "workspace", "members", _])
    {
        goto_member_pyprojects(
            workspace_document_tree,
            accessors,
            workspace_pyproject_toml_path,
            toml_version,
            encoding,
        )
        .map(|locations| locations.into_iter().filter_map(Into::into).collect_vec())
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn goto_workspace_member(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    converter: SpanConverter<'_, '_>,
    jump_to_package: bool,
) -> Result<Option<tombi_extension::Location>, tower_lsp::jsonrpc::Error> {
    debug_assert!(
        matches_accessors!(accessors, ["tool", "uv", "sources", _])
            || matches_accessors!(accessors, ["tool", "uv", "sources", _, "workspace"])
    );

    let package_name = if let tombi_schema_store::Accessor::Key(key) = &accessors[3] {
        key
    } else {
        return Ok(None);
    };
    if accessors.len() == 4
        && let Some((_, tombi_document_tree_syntax::Value::Table(table))) =
            dig_accessors(document_tree, &accessors[..4])
        && !table.contains_key("workspace")
    {
        return Ok(None);
    }

    Ok(find_workspace_pyproject_toml(
        pyproject_toml_path,
        toml_version,
        converter.encoding(),
        |workspace_pyproject_toml_path,
         _,
         workspace_pyproject_toml_document_tree,
         workspace_converter| {
            let (package_location, member_span) = find_member_project_toml(
                package_name,
                workspace_pyproject_toml_document_tree,
                &workspace_pyproject_toml_path,
                toml_version,
                workspace_converter.encoding(),
            )?;

            if jump_to_package {
                let package_pyproject_toml_uri =
                    tombi_uri::Uri::from_file_path(&package_location.pyproject_toml_path).ok()?;

                Some(tombi_extension::Location {
                    uri: package_pyproject_toml_uri,
                    range: Some(package_location.package_name_key_range),
                })
            } else {
                let workspace_pyproject_toml_uri =
                    tombi_uri::Uri::from_file_path(&workspace_pyproject_toml_path).ok()?;

                Some(tombi_extension::Location {
                    uri: workspace_pyproject_toml_uri,
                    range: Some(workspace_converter.range(member_span)),
                })
            }
        },
    )
    .flatten())
}

pub(crate) fn goto_member_pyprojects(
    workspace_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[tombi_schema_store::Accessor],
    workspace_pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    encoding: EncodingKind,
) -> Result<Vec<PackageLocation>, tower_lsp::jsonrpc::Error> {
    let member_patterns = extract_member_patterns(workspace_document_tree, accessors);
    if member_patterns.is_empty() {
        return Ok(Vec::new());
    }

    let Some(workspace_dir_path) = workspace_pyproject_toml_path.parent() else {
        return Ok(Vec::new());
    };

    let exclude_patterns = extract_exclude_patterns(workspace_document_tree);

    let mut locations = Vec::new();
    for (_, pyproject_toml_path) in
        find_pyproject_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some(package_name_key_range) = with_pyproject_toml(
            &pyproject_toml_path,
            toml_version,
            encoding,
            |_, member_document_tree, member_converter| {
                get_project_name(member_document_tree)
                    .map(|package_name| member_converter.range(package_name.unquoted_span()))
            },
        )
        .flatten() else {
            continue;
        };

        locations.push(PackageLocation {
            pyproject_toml_path,
            package_name_key_range,
        });
    }

    Ok(locations)
}

/// Finds the member of the workspace named `package_name`.
///
/// The returned span is the member pattern that matched in the workspace document,
/// and the range of the package name is counted in `encoding`.
pub(crate) fn find_member_project_toml(
    package_name: &str,
    workspace_pyproject_toml_document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    workspace_pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    encoding: EncodingKind,
) -> Option<(PackageLocation, tombi_text::Span)> {
    let workspace_dir_path = workspace_pyproject_toml_path.parent()?;

    let member_patterns = extract_member_patterns(workspace_pyproject_toml_document_tree, &[]);
    let exclude_patterns = extract_exclude_patterns(workspace_pyproject_toml_document_tree);

    for (member_item, package_project_toml_path) in
        find_pyproject_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let Some(package_name_key_range) = with_pyproject_toml(
            &package_project_toml_path,
            toml_version,
            encoding,
            |_, package_project_toml_document_tree, package_converter| {
                get_project_name(package_project_toml_document_tree)
                    .filter(|name| name.value() == package_name)
                    .map(|name| package_converter.range(name.unquoted_span()))
            },
        )
        .flatten() else {
            continue;
        };

        return Some((
            PackageLocation {
                pyproject_toml_path: package_project_toml_path,
                package_name_key_range,
            },
            member_item.unquoted_span(),
        ));
    }

    None
}
