use tombi_config::TomlVersion;
use tombi_document_tree_syntax::{Value, dig_accessors, dig_keys};
use tombi_extension::{HoverMetadata, HoverTextChange, append_latest_version};
use tombi_schema_store::Accessor;

use crate::{
    channels::query_scope,
    conda::fetch_package_versions,
    dependency::{channel_dependency_source, find_conda_dependency},
    is_pixi_toml,
};

pub async fn hover(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    offset: tombi_text::Offset,
    _toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<HoverMetadata>, tower_lsp_server::jsonrpc::Error> {
    if !is_pixi_toml(text_document_uri) {
        return Ok(None);
    }

    let Some(dependency) = find_conda_dependency(accessors) else {
        return Ok(None);
    };

    if !is_hovering_dependency(document_tree, accessors, dependency.is_version_key, offset) {
        return Ok(None);
    }

    let Some(inline_channel) = channel_dependency_source(document_tree, &dependency) else {
        return Ok(None);
    };

    let Some(manifest_dir) = text_document_uri
        .to_file_path()
        .ok()
        .and_then(|path| path.parent().map(ToOwned::to_owned))
    else {
        return Ok(None);
    };

    let scope = query_scope(document_tree, &dependency, inline_channel, &manifest_dir);
    let Some(latest_version) =
        fetch_package_versions(dependency.package_name, &scope, offline, cache_options)
            .await
            .and_then(|versions| versions.into_iter().next())
    else {
        return Ok(None);
    };

    Ok(Some(HoverMetadata {
        title: None,
        description: append_latest_version(None, Some(latest_version)).map(HoverTextChange::Append),
    }))
}

/// Whether the cursor is on the package name key, its version string, or its `version` key/value.
fn is_hovering_dependency(
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    accessors: &[Accessor],
    is_version_key: bool,
    offset: tombi_text::Offset,
) -> bool {
    let Some(keys) = accessors
        .iter()
        .map(Accessor::as_key)
        .collect::<Option<Vec<_>>>()
    else {
        return false;
    };
    let Some((key, value)) = dig_keys(document_tree, &keys) else {
        return false;
    };

    if is_version_key {
        return key.span().contains_inclusive(offset) || value.span().contains_inclusive(offset);
    }

    key.span().contains_inclusive(offset)
        || matches!(
            dig_accessors(document_tree, accessors),
            Some((_, Value::String(version))) if version.span().contains_inclusive(offset)
        )
}
