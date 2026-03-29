use std::str::FromStr;

use pep508_rs::{Requirement, VerbatimUrl, VersionOrUrl};
use tombi_document_tree::dig_keys;
use tombi_schema_store::{Accessor, matches_accessors};

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

    collect_dependency_requirements_from_array_path(
        document_tree,
        &["tool", "uv", "dev-dependencies"],
        &mut dependency_requirements,
    );
    collect_dependency_requirements_from_array_path(
        document_tree,
        &["tool", "uv", "constraint-dependencies"],
        &mut dependency_requirements,
    );
    collect_dependency_requirements_from_array_path(
        document_tree,
        &["tool", "uv", "override-dependencies"],
        &mut dependency_requirements,
    );
    collect_dependency_requirements_from_array_path(
        document_tree,
        &["tool", "uv", "build-constraint-dependencies"],
        &mut dependency_requirements,
    );

    dependency_requirements
}

pub(crate) fn get_dependency_accessors(accessors: &[Accessor]) -> Option<&[Accessor]> {
    if matches_accessors!(accessors, ["project", "dependencies", _]) {
        Some(&accessors[..3])
    } else if matches_accessors!(accessors, ["project", "optional-dependencies", _, _]) {
        Some(&accessors[..4])
    } else if matches_accessors!(accessors, ["dependency-groups", _, _]) {
        Some(&accessors[..3])
    } else if matches_accessors!(accessors, ["tool", "uv", "dev-dependencies", _]) {
        Some(&accessors[..4])
    } else if matches_accessors!(accessors, ["tool", "uv", "constraint-dependencies", _]) {
        Some(&accessors[..4])
    } else if matches_accessors!(accessors, ["tool", "uv", "override-dependencies", _]) {
        Some(&accessors[..4])
    } else if matches_accessors!(
        accessors,
        ["tool", "uv", "build-constraint-dependencies", _]
    ) {
        Some(&accessors[..4])
    } else {
        None
    }
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
}
