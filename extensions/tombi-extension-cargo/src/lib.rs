mod code_action;
mod completion;
mod document_link;
mod goto_declaration;
mod goto_definition;
mod hover;
mod manifest;
mod workspace;

pub use code_action::{CodeActionRefactorRewriteName, code_action};
pub use completion::completion;
pub use document_link::{DocumentLinkToolTip, document_link};
pub use goto_declaration::goto_declaration;
pub use goto_definition::goto_definition;
pub use hover::hover;

pub(crate) use manifest::{
    CrateLocation, find_path_crate_cargo_toml, find_workspace_cargo_toml,
    get_uri_relative_to_cargo_toml, get_workspace_path, load_cargo_toml,
};
pub(crate) use workspace::{
    find_package_cargo_toml_paths, goto_definition_for_crate_cargo_toml,
    goto_definition_for_workspace_cargo_toml, sanitize_dependency_key,
};

pub(crate) enum CargoNavigationFeature {
    Dependency,
    Member,
    Path,
}

pub(crate) fn classify_cargo_navigation_feature(
    accessors: &[tombi_schema_store::Accessor],
) -> CargoNavigationFeature {
    if matches!(
        accessors.last(),
        Some(tombi_schema_store::Accessor::Key(key)) if key == "path"
    ) {
        CargoNavigationFeature::Path
    } else if matches!(
        accessors.first(),
        Some(tombi_schema_store::Accessor::Key(key)) if key == "workspace"
    ) {
        CargoNavigationFeature::Member
    } else {
        CargoNavigationFeature::Dependency
    }
}
