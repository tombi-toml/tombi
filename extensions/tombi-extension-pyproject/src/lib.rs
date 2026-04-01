mod code_action;
mod completion;
mod dependency;
mod did_open;
mod document_link;
mod goto_declaration;
mod goto_definition;
mod hover;
mod inlay_hint;
mod manifest;
mod workspace;

pub use code_action::code_action;
pub use completion::completion;
pub use did_open::did_open;
pub use document_link::document_link;
pub use goto_declaration::goto_declaration;
pub use goto_definition::goto_definition;
pub use hover::hover;
pub use inlay_hint::inlay_hint;

pub(crate) use dependency::{
    DependencyRequirement, UV_DEPENDENCY_KEYS,
    collect_all_dependency_requirements_from_document_tree,
    collect_dependency_requirements_from_document_tree, find_dependency_group_key,
    get_dependency_accessors, include_group_locations,
    parse_dependency_requirement,
    parse_requirement,
};
pub(crate) use manifest::{
    PackageLocation, find_workspace_pyproject_toml, get_project_name,
    load_pyproject_toml_document_tree, resolve_member_pyproject_toml_path,
};
use tombi_schema_store::matches_accessors;
pub(crate) use workspace::{
    extract_exclude_patterns, extract_member_patterns, find_member_project_toml,
    find_pyproject_toml_paths, goto_definition_for_member_pyproject_toml,
    goto_definition_for_workspace_pyproject_toml, goto_member_pyprojects,
};

pub(crate) enum PyprojectNavigationFeature {
    Dependency,
    Member,
    Path,
}

pub(crate) fn classify_pyproject_navigation_feature(
    accessors: &[tombi_schema_store::Accessor],
) -> PyprojectNavigationFeature {
    if matches!(
        accessors.last(),
        Some(tombi_schema_store::Accessor::Key(key)) if key == "path"
    ) {
        PyprojectNavigationFeature::Path
    } else if matches_accessors!(
        accessors[..accessors.len().min(3)],
        ["tool", "uv", "workspace"]
    ) || matches_accessors!(
        accessors[..accessors.len().min(3)],
        ["tool", "uv", "sources"]
    ) {
        PyprojectNavigationFeature::Member
    } else {
        PyprojectNavigationFeature::Dependency
    }
}
