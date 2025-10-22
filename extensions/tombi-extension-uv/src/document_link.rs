use std::borrow::Cow;
use std::str::FromStr;

use pep508_rs::{Requirement, VerbatimUrl};
use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;

use crate::{find_member_project_toml, find_workspace_pyproject_toml, goto_member_pyprojects};

pub enum DocumentLinkToolTip {
    PyprojectToml,
    PyprojectTomlFirstMember,
    WorkspacePyprojectToml,
    PyPI,
}

impl From<&DocumentLinkToolTip> for &'static str {
    #[inline]
    fn from(val: &DocumentLinkToolTip) -> Self {
        match val {
            DocumentLinkToolTip::PyprojectToml => "Open pyproject.toml",
            DocumentLinkToolTip::PyprojectTomlFirstMember => "Open first pyproject.toml in members",
            DocumentLinkToolTip::WorkspacePyprojectToml => "Open Workspace pyproject.toml",
            DocumentLinkToolTip::PyPI => "Open PyPI Package",
        }
    }
}

impl From<DocumentLinkToolTip> for &'static str {
    #[inline]
    fn from(val: DocumentLinkToolTip) -> Self {
        (&val).into()
    }
}

impl From<DocumentLinkToolTip> for Cow<'static, str> {
    #[inline]
    fn from(val: DocumentLinkToolTip) -> Self {
        Cow::Borrowed(val.into())
    }
}

impl std::fmt::Display for DocumentLinkToolTip {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(self))
    }
}

pub async fn document_link(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    toml_version: TomlVersion,
) -> Result<Option<Vec<tombi_extension::DocumentLink>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is Cargo.toml
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    let Some(pyproject_toml_path) = text_document_uri.to_file_path().ok() else {
        return Ok(None);
    };

    let mut document_links = vec![];

    if let Some((_, tombi_document_tree::Value::Table(workspace))) =
        dig_keys(document_tree, &["tool", "uv", "workspace"])
    {
        document_links.extend(document_link_for_workspace_pyproject_toml(
            document_tree,
            workspace,
            &pyproject_toml_path,
            toml_version,
        )?);
    }

    // Collect tool.uv.sources information
    let uv_sources = if let Some((_, tombi_document_tree::Value::Table(sources))) =
        dig_keys(document_tree, &["tool", "uv", "sources"])
    {
        for (package_name_key, source) in sources.key_values() {
            document_links.extend(document_link_for_member_pyproject_toml(
                package_name_key,
                source,
                &pyproject_toml_path,
                toml_version,
            )?);
        }

        Some(sources)
    } else {
        None
    };

    // Handle [project.dependencies] section
    if let Some((_, tombi_document_tree::Value::Array(dependencies))) =
        dig_keys(document_tree, &["project", "dependencies"])
    {
        document_links.extend(document_link_for_project_dependencies(
            dependencies,
            uv_sources,
            &pyproject_toml_path,
            toml_version,
        )?);
    }

    // Handle [project.optional-dependencies] section
    if let Some((_, tombi_document_tree::Value::Table(optional_dependencies))) =
        dig_keys(document_tree, &["project", "optional-dependencies"])
    {
        document_links.extend(document_link_for_optional_dependencies(
            optional_dependencies,
            uv_sources,
            &pyproject_toml_path,
            toml_version,
        )?);
    }

    // Handle [dependency-groups] section
    if let Some((_, tombi_document_tree::Value::Table(dependency_groups))) =
        dig_keys(document_tree, &["dependency-groups"])
    {
        document_links.extend(document_link_for_dependency_groups(
            dependency_groups,
            uv_sources,
            &pyproject_toml_path,
            toml_version,
        )?);
    }

    if document_links.is_empty() {
        return Ok(None);
    }

    Ok(Some(document_links))
}

fn document_link_for_workspace_pyproject_toml(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace: &tombi_document_tree::Table,
    workspace_pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let Some(tombi_document_tree::Value::Array(members)) = workspace.get("members") else {
        return Ok(Vec::with_capacity(0));
    };

    let mut total_document_links = vec![];
    for (index, member) in members.values().iter().enumerate() {
        let tombi_document_tree::Value::String(member) = member else {
            continue;
        };

        let Ok(member_paskage_locations) = goto_member_pyprojects(
            workspace_document_tree,
            &[
                tombi_schema_store::Accessor::Key("tool".to_string()),
                tombi_schema_store::Accessor::Key("uv".to_string()),
                tombi_schema_store::Accessor::Key("workspace".to_string()),
                tombi_schema_store::Accessor::Key("members".to_string()),
                tombi_schema_store::Accessor::Index(index),
            ],
            workspace_pyproject_toml_path,
            toml_version,
        ) else {
            continue;
        };

        let mut member_document_links =
            member_paskage_locations.into_iter().filter_map(|location| {
                let Ok(member_pyproject_toml_uri) =
                    tombi_uri::Uri::from_file_path(&location.pyproject_toml_path)
                else {
                    return None;
                };
                Some(tombi_extension::DocumentLink {
                    target: member_pyproject_toml_uri,
                    range: member.unquoted_range(),
                    tooltip: DocumentLinkToolTip::PyprojectTomlFirstMember.into(),
                })
            });

        match member_document_links.size_hint() {
            (_, Some(n)) if n > 0 => {
                if let Some(mut document_link) = member_document_links.next() {
                    if n == 1 {
                        document_link.tooltip = DocumentLinkToolTip::PyprojectToml.into();
                    }
                    total_document_links.push(document_link);
                }
            }
            _ => {}
        }
    }

    Ok(total_document_links)
}

fn document_link_for_member_pyproject_toml(
    package_name_key: &tombi_document_tree::Key,
    source: &tombi_document_tree::Value,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let tombi_document_tree::Value::Table(source) = source else {
        return Ok(Vec::with_capacity(0));
    };

    let Some((workspace_pyproject_toml_path, _, workspace_pyproject_toml_document_tree)) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)
    else {
        return Ok(Vec::with_capacity(0));
    };

    let Ok(workspace_pyproject_toml_uri) =
        tombi_uri::Uri::from_file_path(&workspace_pyproject_toml_path)
    else {
        return Ok(Vec::with_capacity(0));
    };

    let mut document_links = vec![];
    if let Some((workspace_key, tombi_document_tree::Value::Boolean(is_workspace))) =
        source.get_key_value("workspace")
    {
        if is_workspace.value() {
            if let Some((member_project_toml_path, _)) = find_member_project_toml(
                &package_name_key.value,
                &workspace_pyproject_toml_document_tree,
                &workspace_pyproject_toml_path,
                toml_version,
            ) {
                if let Ok(member_project_toml_uri) =
                    tombi_uri::Uri::from_file_path(&member_project_toml_path)
                {
                    document_links.push(tombi_extension::DocumentLink {
                        target: member_project_toml_uri,
                        range: package_name_key.unquoted_range(),
                        tooltip: DocumentLinkToolTip::PyprojectToml.into(),
                    });
                }
                document_links.push(tombi_extension::DocumentLink {
                    target: workspace_pyproject_toml_uri.clone(),
                    range: workspace_key.range() + is_workspace.range(),
                    tooltip: DocumentLinkToolTip::WorkspacePyprojectToml.into(),
                });
            }
        }
    }

    Ok(document_links)
}

fn document_link_for_project_dependencies(
    dependencies: &tombi_document_tree::Array,
    uv_sources: Option<&tombi_document_tree::Table>,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut document_links = Vec::with_capacity(dependencies.len());

    for dep_value in dependencies.values() {
        if let tombi_document_tree::Value::String(dep_spec) = dep_value {
            // Parse the PEP 508 requirement specification
            if let Ok(requirement) = Requirement::<VerbatimUrl>::from_str(dep_spec.value()) {
                // Extract package name from the requirement
                let package_name = &requirement.name;

                // Check if this package is in tool.uv.sources
                let is_local_source =
                    uv_sources.is_some_and(|sources| sources.contains_key(package_name.as_ref()));

                if is_local_source {
                    // For packages in tool.uv.sources, create links to local pyproject.toml
                    if let Some(sources) = uv_sources {
                        if let Some((package_key, source_value)) =
                            sources.get_key_value(package_name.as_ref())
                        {
                            // Try to create document links for the local source
                            if let Ok(links) = document_link_for_member_pyproject_toml(
                                package_key,
                                source_value,
                                pyproject_toml_path,
                                toml_version,
                            ) {
                                for link in links {
                                    if link.tooltip
                                        == Into::<&'static str>::into(
                                            DocumentLinkToolTip::PyprojectToml,
                                        )
                                    {
                                        // Update the range to point to the dependency spec
                                        document_links.push(tombi_extension::DocumentLink {
                                            target: link.target,
                                            range: dep_spec.unquoted_range(),
                                            tooltip: link.tooltip,
                                        });
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Create PyPI URL for external packages
                    if let Ok(pypi_uri) = tombi_uri::Uri::from_str(&format!(
                        "https://pypi.org/project/{package_name}/"
                    )) {
                        document_links.push(tombi_extension::DocumentLink {
                            target: pypi_uri,
                            range: dep_spec.unquoted_range(),
                            tooltip: DocumentLinkToolTip::PyPI.into(),
                        });
                    }
                }
            }
        }
    }

    Ok(document_links)
}

fn document_link_for_dependency_groups(
    dependency_groups: &tombi_document_tree::Table,
    uv_sources: Option<&tombi_document_tree::Table>,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut document_links = Vec::new();

    // Iterate through each dependency group
    for dependency_group in dependency_groups.values() {
        if let tombi_document_tree::Value::Array(dependencies) = dependency_group {
            // Process each dependency in the group using the same logic as project.dependencies
            document_links.extend(document_link_for_project_dependencies(
                dependencies,
                uv_sources,
                pyproject_toml_path,
                toml_version,
            )?);
        }
    }

    Ok(document_links)
}

fn document_link_for_optional_dependencies(
    optional_dependencies: &tombi_document_tree::Table,
    uv_sources: Option<&tombi_document_tree::Table>,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut document_links = Vec::new();

    // Iterate through each optional dependency group
    for option in optional_dependencies.values() {
        if let tombi_document_tree::Value::Array(dependencies) = option {
            // Process each dependency in the group using the same logic as project.dependencies
            document_links.extend(document_link_for_project_dependencies(
                dependencies,
                uv_sources,
                pyproject_toml_path,
                toml_version,
            )?);
        }
    }

    Ok(document_links)
}
