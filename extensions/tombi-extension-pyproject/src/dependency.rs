use std::str::FromStr;

use pep508_rs::{Requirement, VerbatimUrl, VersionOrUrl};
use tombi_document_tree::dig_keys;

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

    if let Some((_, tombi_document_tree::Value::Array(dep_array))) =
        dig_keys(document_tree, &["project", "dependencies"])
    {
        dependency_requirements.extend(collect_dependency_requirements_from_values::<'a>(
            dep_array.iter(),
        ));
    }
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

    dependency_requirements
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
