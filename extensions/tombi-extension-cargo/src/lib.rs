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
