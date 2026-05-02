use tombi_schema_store::{Accessor, matches_accessors};

#[inline]
pub(crate) fn is_project_name_accessors(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["project", "name"])
}

#[inline]
pub(crate) fn is_dependency_name_accessors(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["project", "dependencies", _])
        || matches_accessors!(accessors, ["project", "optional-dependencies", _, _])
        || matches_accessors!(accessors, ["dependency-groups", _, _])
}

#[inline]
pub(crate) fn is_dependency_group_name_accessors(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["dependency-groups", _])
}

#[inline]
pub(crate) fn is_dependency_groups_include_group_accessors(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["dependency-groups", _, _, "include-group"])
}

#[inline]
pub(crate) fn has_uv_sources_accessors(accessors: &[Accessor]) -> bool {
    matches_accessors!(
        accessors[..accessors.len().min(3)],
        ["tool", "uv", "sources"]
    )
}

#[inline]
pub(crate) fn is_uv_sources_accessors(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["tool", "uv", "sources", _])
}

#[inline]
pub(crate) fn is_uv_source_workspace_accessors(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["tool", "uv", "sources", _, "workspace"])
}

#[inline]
pub(crate) fn is_uv_source_path_accessors(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["tool", "uv", "sources", _, "path"])
}

#[inline]
pub(crate) fn is_uv_workspace_accessors(accessors: &[Accessor]) -> bool {
    matches_accessors!(
        accessors[..accessors.len().min(3)],
        ["tool", "uv", "workspace"]
    )
}
