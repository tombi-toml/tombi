use tombi_document_tree::{Value, dig_accessors};
use tombi_schema_store::{Accessor, matches_accessors};

#[inline]
pub(crate) fn is_package_name_accessor(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["package", "name"])
}

#[inline]
pub(crate) fn is_feature_key_accessor(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["features", _])
}

#[inline]
pub(crate) fn is_dependency_accessor(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["dependencies", _])
        || matches_accessors!(accessors, ["dev-dependencies", _])
        || matches_accessors!(accessors, ["build-dependencies", _])
        || matches_accessors!(accessors, ["target", _, "dependencies", _])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _])
        || matches_accessors!(accessors, ["target", _, "build-dependencies", _])
}

#[inline]
pub(crate) fn is_workspace_dependency_accessor(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["workspace", "dependencies", _])
}

#[inline]
pub(crate) fn is_any_dependency_accessor(accessors: &[Accessor]) -> bool {
    is_workspace_dependency_accessor(accessors) || is_dependency_accessor(accessors)
}

#[inline]
pub(crate) fn is_dependency_path_accessor(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["dependencies", _, "path"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "path"])
        || matches_accessors!(accessors, ["build-dependencies", _, "path"])
        || matches_accessors!(accessors, ["target", _, "dependencies", _, "path"])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, "path"])
        || matches_accessors!(accessors, ["target", _, "build-dependencies", _, "path"])
}

#[inline]
pub(crate) fn is_any_dependency_path_accessor(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["workspace", "dependencies", _, "path"])
        || is_dependency_path_accessor(accessors)
}

#[inline]
pub(crate) fn is_workspace_key_accessor(accessors: &[Accessor]) -> bool {
    matches!(accessors.last(), Some(Accessor::Key(key)) if key == "workspace")
}

#[inline]
pub(crate) fn is_optional_dependency_accessor(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["dependencies", _, "optional"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "optional"])
        || matches_accessors!(accessors, ["build-dependencies", _, "optional"])
        || matches_accessors!(accessors, ["target", _, "dependencies", _, "optional"])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, "optional"])
        || matches_accessors!(
            accessors,
            ["target", _, "build-dependencies", _, "optional"]
        )
}

#[inline]
pub(crate) fn is_workspace_definition_accessor(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["workspace", "dependencies", _])
        || matches_accessors!(accessors, ["workspace", "dependencies", _, "path"])
        || matches_accessors!(accessors, ["workspace", "members"])
        || matches_accessors!(accessors, ["workspace", "members", _])
        || matches_accessors!(accessors, ["workspace", "default-members"])
        || matches_accessors!(accessors, ["workspace", "default-members", _])
}

#[inline]
pub(crate) fn is_workspace_flag_accessor(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["dependencies", _, "workspace"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "workspace"])
        || matches_accessors!(accessors, ["build-dependencies", _, "workspace"])
        || matches_accessors!(accessors, ["target", _, "dependencies", _, "workspace"])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, "workspace"])
        || matches_accessors!(
            accessors,
            ["target", _, "build-dependencies", _, "workspace"]
        )
}

#[inline]
pub(crate) fn is_workspace_managed_dependency_accessor(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
) -> bool {
    is_dependency_accessor(accessors)
        && matches!(
            dig_accessors(document_tree, accessors),
            Some((_, Value::Table(table)))
                if matches!(
                    table.get("workspace"),
                    Some(Value::Boolean(has_workspace)) if has_workspace.value()
                )
        )
}

#[inline]
pub(crate) fn dependency_parent_accessors(accessors: &[Accessor]) -> &[Accessor] {
    &accessors[..accessors.len().saturating_sub(1)]
}
