mod cargo_lock;
mod cargo_toml;
mod code_action;
mod completion;
mod did_open;
mod document_link;
mod feature_navigation;
mod goto_declaration;
mod goto_definition;
mod hover;
mod inlay_hint;
mod references;
mod workspace;

pub use code_action::{CodeActionRefactorRewriteName, code_action};
pub use completion::completion;
pub use did_open::did_open;
pub use document_link::{DocumentLinkToolTip, document_link};
pub use goto_declaration::goto_declaration;
pub use goto_definition::goto_definition;
pub use hover::hover;
pub use inlay_hint::inlay_hint;
pub(crate) use references::package_name_reference_locations;
pub use references::references;

pub(crate) use cargo_toml::{
    CrateLocation, dependency_package_name, find_cargo_toml, get_uri_relative_to_cargo_toml,
    load_cargo_toml,
};
pub(crate) use feature_navigation::{
    CargoTargetLocation, collect_feature_usage_locations,
    collect_feature_usage_locations_in_manifest, dependency_feature_string_context,
    feature_key_at_accessors, feature_table_string_at_accessors,
    feature_usage_target_for_feature_key, feature_usage_target_for_optional_dependency,
    optional_dependency_value_at_accessors, resolve_dependency_feature_string,
    resolve_feature_table_string,
};
pub(crate) use workspace::{
    canonicalize_or_original, find_package_cargo_toml_paths, find_workspace_cargo_toml,
    get_workspace_cargo_toml_path, goto_declaration_for_crate_cargo_toml,
    goto_definition_for_workspace_cargo_toml, goto_workspace_member_crates,
    load_cargo_toml_document_tree, load_workspace_cargo_toml, sanitize_dependency_key,
    workspace_dependency_usage_locations,
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
        accessors,
        [
            tombi_schema_store::Accessor::Key(first),
            tombi_schema_store::Accessor::Key(second),
            ..
        ] if *first == "workspace" && (*second == "members" || *second == "default-members")
    ) {
        CargoNavigationFeature::Member
    } else {
        CargoNavigationFeature::Dependency
    }
}

#[cfg(test)]
mod tests {
    use super::{CargoNavigationFeature, classify_cargo_navigation_feature};

    fn key(value: &str) -> tombi_schema_store::Accessor {
        tombi_schema_store::Accessor::Key(value.to_string())
    }

    #[test]
    fn classify_workspace_members_as_member_feature() {
        let feature = classify_cargo_navigation_feature(&[key("workspace"), key("members")]);

        assert!(matches!(feature, CargoNavigationFeature::Member));
    }

    #[test]
    fn classify_workspace_dependencies_as_dependency_feature() {
        let feature =
            classify_cargo_navigation_feature(&[key("workspace"), key("dependencies"), key("foo")]);

        assert!(matches!(feature, CargoNavigationFeature::Dependency));
    }
}
