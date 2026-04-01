use std::str::FromStr;

use pep508_rs::{Requirement, VerbatimUrl, VersionOrUrl};
use tombi_document_tree::{Value, dig_keys};
use tombi_schema_store::{Accessor, matches_accessors};

pub(crate) const UV_DEPENDENCY_KEYS: &[&str] = &[
    "dev-dependencies",
    "constraint-dependencies",
    "override-dependencies",
    "build-constraint-dependencies",
];

#[derive(Debug, Clone)]
pub(crate) struct DependencyRequirement<'a> {
    pub(crate) dependency: &'a tombi_document_tree::String,
    pub(crate) requirement: Requirement<VerbatimUrl>,
}

impl<'a> DependencyRequirement<'a> {
    #[inline]
    pub(crate) fn version_or_url(&self) -> Option<&VersionOrUrl<VerbatimUrl>> {
        self.requirement.version_or_url.as_ref()
    }
}

pub(crate) fn parse_requirement(dependency: &str) -> Option<Requirement<VerbatimUrl>> {
    match Requirement::<VerbatimUrl>::from_str(dependency) {
        Ok(requirement) => Some(requirement),
        Err(e) => {
            log::debug!(
                "Failed to parse PEP 508 dependency string: dependency={:?}, error={:?}",
                dependency,
                e
            );
            None
        }
    }
}

pub(crate) fn parse_dependency_requirement<'a>(
    dependency: &'a tombi_document_tree::String,
) -> Option<DependencyRequirement<'a>> {
    parse_requirement(dependency.value()).map(|requirement| DependencyRequirement {
        requirement,
        dependency,
    })
}

pub(crate) fn collect_dependency_requirements_from_document_tree<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
) -> Vec<DependencyRequirement<'a>> {
    let mut dependency_requirements = Vec::new();

    collect_standard_dependency_requirements(document_tree, &mut dependency_requirements);

    dependency_requirements
}

pub(crate) fn collect_all_dependency_requirements_from_document_tree<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
) -> Vec<DependencyRequirement<'a>> {
    let mut dependency_requirements =
        collect_dependency_requirements_from_document_tree(document_tree);

    for key in UV_DEPENDENCY_KEYS {
        collect_dependency_requirements_from_array_path(
            document_tree,
            &["tool", "uv", key],
            &mut dependency_requirements,
        );
    }

    dependency_requirements
}

pub(crate) fn get_dependency_accessors(accessors: &[Accessor]) -> Option<&[Accessor]> {
    if matches_accessors!(accessors, ["project", "dependencies", _]) {
        Some(&accessors[..3])
    } else if matches_accessors!(accessors, ["project", "optional-dependencies", _, _]) {
        Some(&accessors[..4])
    } else if matches_accessors!(accessors, ["dependency-groups", _, _]) {
        Some(&accessors[..3])
    } else if is_uv_dependency_accessor(accessors) {
        Some(&accessors[..4])
    } else {
        None
    }
}

fn is_uv_dependency_accessor(accessors: &[Accessor]) -> bool {
    if accessors.len() != 4 {
        return false;
    }
    matches!(
        (&accessors[0], &accessors[1], &accessors[3]),
        (Accessor::Key(a), Accessor::Key(b), Accessor::Index(_))
        if a == "tool" && b == "uv"
    ) && matches!(
        &accessors[2],
        Accessor::Key(key) if UV_DEPENDENCY_KEYS.contains(&key.as_str())
    )
}

fn collect_standard_dependency_requirements<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    dependency_requirements: &mut Vec<DependencyRequirement<'a>>,
) {
    collect_dependency_requirements_from_array_path(
        document_tree,
        &["project", "dependencies"],
        dependency_requirements,
    );

    if let Some((_, tombi_document_tree::Value::Table(dep_group))) =
        dig_keys(document_tree, &["project", "optional-dependencies"])
    {
        for value in dep_group.values() {
            if let tombi_document_tree::Value::Array(dep_array) = value {
                dependency_requirements.extend(collect_dependency_requirements_from_values(
                    dep_array.iter(),
                ));
            }
        }
    }

    if let Some((_, tombi_document_tree::Value::Table(dep_group))) =
        dig_keys(document_tree, &["dependency-groups"])
    {
        for value in dep_group.values() {
            if let tombi_document_tree::Value::Array(dep_array) = value {
                dependency_requirements.extend(collect_dependency_requirements_from_values(
                    dep_array.iter(),
                ));
            }
        }
    }
}

fn collect_dependency_requirements_from_array_path<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    path: &[&str],
    dependency_requirements: &mut Vec<DependencyRequirement<'a>>,
) {
    if let Some((_, tombi_document_tree::Value::Array(dep_array))) = dig_keys(document_tree, path) {
        dependency_requirements.extend(collect_dependency_requirements_from_values(
            dep_array.iter(),
        ));
    }
}

fn collect_dependency_requirements_from_values<'a>(
    dependencies: impl Iterator<Item = &'a tombi_document_tree::Value>,
) -> Vec<DependencyRequirement<'a>> {
    dependencies
        .filter_map(|value| {
            if let tombi_document_tree::Value::String(dep_str) = value {
                parse_requirement(dep_str.value()).map(|requirement| DependencyRequirement {
                    requirement,
                    dependency: dep_str,
                })
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn find_dependency_group_key<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    group_name: &str,
) -> Option<&'a tombi_document_tree::Key> {
    let (_, Value::Table(dependency_groups)) = dig_keys(document_tree, &["dependency-groups"])?
    else {
        return None;
    };

    let (group_key, _) = dependency_groups.get_key_value(group_name)?;
    Some(group_key)
}

pub(crate) fn include_group_locations(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    pyproject_toml_path: &std::path::Path,
) -> Result<Vec<tombi_extension::DefinitionLocation>, tower_lsp::jsonrpc::Error> {
    let Some(tombi_schema_store::Accessor::Key(group_name)) = accessors.get(1) else {
        return Ok(Vec::with_capacity(0));
    };

    let Ok(uri) = tombi_uri::Uri::from_file_path(pyproject_toml_path) else {
        return Ok(Vec::with_capacity(0));
    };

    Ok(collect_include_group_values(document_tree, group_name)
        .into_iter()
        .map(|include_group| tombi_extension::DefinitionLocation {
            uri: uri.clone(),
            range: include_group.unquoted_range(),
        })
        .collect())
}

pub(crate) fn collect_include_group_values<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    group_name: &str,
) -> Vec<&'a tombi_document_tree::String> {
    let Some((_, Value::Table(dependency_groups))) =
        dig_keys(document_tree, &["dependency-groups"])
    else {
        return Vec::with_capacity(0);
    };

    let mut include_group_values = Vec::new();
    for dependency_group in dependency_groups.values() {
        let Value::Array(dependencies) = dependency_group else {
            continue;
        };

        for dependency in dependencies.values() {
            let Value::Table(include_group_table) = dependency else {
                continue;
            };

            let Some((_, Value::String(include_group))) =
                include_group_table.get_key_value("include-group")
            else {
                continue;
            };

            if include_group.value() == group_name {
                include_group_values.push(include_group);
            }
        }
    }

    include_group_values
}

#[cfg(test)]
mod tests {
    use tombi_ast::AstNode;
    use tombi_config::TomlVersion;
    use tombi_document_tree::TryIntoDocumentTree;

    use super::*;

    fn parse_document_tree(source: &str) -> tombi_document_tree::DocumentTree {
        let root = tombi_ast::Root::cast(tombi_parser::parse(source).into_syntax_node()).unwrap();
        root.try_into_document_tree(TomlVersion::default()).unwrap()
    }

    #[test]
    fn collects_tool_uv_dependency_lists_for_extended_features() {
        let document_tree = parse_document_tree(
            r#"
            [project]
            dependencies = ["requests>=2.0"]

            [tool.uv]
            dev-dependencies = ["ruff>=0.7"]
            constraint-dependencies = ["pytest<9"]
            override-dependencies = ["werkzeug==2.3.0"]
            build-constraint-dependencies = ["setuptools==60.0.0"]
            "#,
        );

        let dependency_names =
            collect_all_dependency_requirements_from_document_tree(&document_tree)
                .into_iter()
                .map(|dependency| dependency.requirement.name.to_string())
                .collect::<Vec<_>>();

        assert_eq!(
            dependency_names,
            vec!["requests", "ruff", "pytest", "werkzeug", "setuptools"]
        );
    }

    #[test]
    fn recognizes_tool_uv_dependency_accessors() {
        let accessors = vec![
            Accessor::Key("tool".to_string()),
            Accessor::Key("uv".to_string()),
            Accessor::Key("override-dependencies".to_string()),
            Accessor::Index(0),
        ];

        assert_eq!(
            get_dependency_accessors(&accessors),
            Some(accessors.as_slice())
        );
    }

    #[test]
    fn finds_dependency_group_key_and_include_group_values() {
        let document_tree = parse_document_tree(
            r#"
            [dependency-groups]
            dev = [{ include-group = "ci" }]
            qa = [{ include-group = "ci" }]
            ci = ["ruff"]
            "#,
        );

        let group_key = find_dependency_group_key(&document_tree, "ci")
            .expect("expected dependency group key to exist");
        let include_group_values = collect_include_group_values(&document_tree, "ci");

        assert_eq!(group_key.value, "ci");
        assert_eq!(
            include_group_values
                .into_iter()
                .map(|include_group| include_group.value())
                .collect::<Vec<_>>(),
            vec!["ci", "ci"]
        );
    }
}
