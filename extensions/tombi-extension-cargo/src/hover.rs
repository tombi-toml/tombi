use std::path::Path;

use serde::Deserialize;
use tombi_config::TomlVersion;
use tombi_document_tree::{Value, dig_accessors, dig_keys};
use tombi_extension::{HoverMetadata, append_latest_version, fetch_cached_remote_json};
use tombi_schema_store::{Accessor, matches_accessors};

use crate::{
    collect_feature_usage_locations, dependency_package_name, feature_key_at_accessors,
    feature_usage_target_for_feature_key, find_cargo_toml, find_workspace_cargo_toml,
    get_workspace_cargo_toml_path, is_any_dependency_accessor, load_cargo_toml,
    sanitize_dependency_key,
};

#[derive(Debug, Deserialize)]
struct CratesIoCrateResponse {
    #[serde(rename = "crate")]
    crate_info: CratesIoCrate,
}

#[derive(Debug, Deserialize)]
struct CratesIoCrate {
    name: Option<String>,
    description: Option<String>,
    max_version: Option<String>,
}

pub async fn hover(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    position: tombi_text::Position,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<HoverMetadata>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }

    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    if let Some(metadata) = feature_key_hover_metadata(
        document_tree,
        accessors,
        position,
        &cargo_toml_path,
        toml_version,
    )
    .await
    {
        return Ok(Some(metadata));
    }

    let Some(dependency_accessors) = get_dependency_accessors(accessors) else {
        return Ok(None);
    };

    if !is_hovering_dependency_key(document_tree, dependency_accessors, position) {
        return Ok(None);
    }

    let Some(Accessor::Key(dependency_key)) = dependency_accessors.last() else {
        return Ok(None);
    };

    let Some((_, dependency_value)) = dig_accessors(document_tree, dependency_accessors) else {
        return Ok(None);
    };

    if let Some(metadata) = resolve_local_dependency_metadata(
        document_tree,
        dependency_accessors,
        dependency_key,
        dependency_value,
        &cargo_toml_path,
        toml_version,
    ) {
        return Ok(Some(metadata));
    }

    if is_unsupported_remote_dependency(dependency_value) {
        return Ok(None);
    }

    let package_name = dependency_package_name(dependency_key, dependency_value);
    fetch_crates_io_metadata(package_name, offline, cache_options).await
}

async fn feature_key_hover_metadata(
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    position: tombi_text::Position,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<HoverMetadata> {
    let feature_key = feature_key_at_accessors(document_tree, accessors)?;
    if !feature_key.range().contains(position) {
        return None;
    }

    let target = feature_usage_target_for_feature_key(cargo_toml_path, accessors)?;
    let usage_locations =
        collect_feature_usage_locations(document_tree, cargo_toml_path, &target, toml_version)
            .await;
    if usage_locations.is_empty() {
        return None;
    }

    Some(HoverMetadata {
        title: None,
        description: Some(render_feature_usage_links(
            document_tree,
            cargo_toml_path,
            usage_locations.as_slice(),
            toml_version,
        )),
    })
}

fn render_feature_usage_links(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    usage_locations: &[crate::CargoTargetLocation],
    toml_version: TomlVersion,
) -> String {
    let project_root = feature_usage_project_root(document_tree, cargo_toml_path, toml_version);
    let mut lines = vec!["Feature references in this project:".to_string()];

    for location in usage_locations {
        let line = location.range.start.line + 1;
        let label = format_feature_usage_label(&project_root, &location.cargo_toml_path, line);

        match tombi_uri::Uri::from_file_path(&location.cargo_toml_path) {
            Ok(mut uri) => {
                uri.set_fragment(Some(&format!("L{line}")));
                lines.push(format!("- [{label}]({uri})"));
            }
            Err(_) => lines.push(format!("- `{label}`")),
        }
    }

    lines.join("\n")
}

fn feature_usage_project_root(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> std::path::PathBuf {
    if document_tree.contains_key("workspace") {
        return crate::canonicalize_or_original(
            cargo_toml_path
                .parent()
                .unwrap_or(cargo_toml_path)
                .to_path_buf(),
        );
    }

    find_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_cargo_toml_path(document_tree),
        toml_version,
    )
    .and_then(|(workspace_cargo_toml_path, _, _)| {
        workspace_cargo_toml_path.parent().map(Path::to_path_buf)
    })
    .map(crate::canonicalize_or_original)
    .unwrap_or_else(|| {
        crate::canonicalize_or_original(
            cargo_toml_path
                .parent()
                .unwrap_or(cargo_toml_path)
                .to_path_buf(),
        )
    })
}

fn format_feature_usage_label(project_root: &Path, cargo_toml_path: &Path, line: u32) -> String {
    let relative_path = cargo_toml_path
        .strip_prefix(project_root)
        .unwrap_or(cargo_toml_path)
        .to_string_lossy()
        .replace('\\', "/");

    format!("{relative_path}:{line}")
}

fn get_dependency_accessors(accessors: &[Accessor]) -> Option<&[Accessor]> {
    if is_any_dependency_accessor(accessors) {
        Some(accessors)
    } else {
        None
    }
}

fn is_hovering_dependency_key(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_accessors: &[Accessor],
    position: tombi_text::Position,
) -> bool {
    let dependency_keys = dependency_accessors
        .iter()
        .map(Accessor::as_key)
        .collect::<Option<Vec<_>>>();
    let Some(dependency_keys) = dependency_keys else {
        return false;
    };
    let Some((dependency_key, _)) = dig_keys(document_tree, &dependency_keys) else {
        return false;
    };

    dependency_key.range().contains(position)
}

fn resolve_local_dependency_metadata(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_accessors: &[Accessor],
    dependency_key: &str,
    dependency_value: &Value,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<HoverMetadata> {
    if let Value::Table(table) = dependency_value {
        if let Some(Value::String(path)) = table.get("path")
            && let Some((resolved_cargo_toml_path, _, _)) =
                find_cargo_toml(cargo_toml_path, Path::new(path.value()), toml_version)
        {
            return load_package_metadata(&resolved_cargo_toml_path, toml_version);
        }

        if let Some(Value::Boolean(workspace)) = table.get("workspace")
            && workspace.value()
        {
            return resolve_workspace_dependency_metadata(
                document_tree,
                dependency_accessors,
                dependency_key,
                cargo_toml_path,
                toml_version,
            );
        }
    }

    None
}

fn resolve_workspace_dependency_metadata(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_accessors: &[Accessor],
    dependency_key: &str,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<HoverMetadata> {
    let (workspace_cargo_toml_path, _, workspace_document_tree) = find_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_cargo_toml_path(document_tree),
        toml_version,
    )?;

    let dependency_kind = if matches_accessors!(dependency_accessors, ["workspace", _, _]) {
        match &dependency_accessors[1] {
            Accessor::Key(key) => key.as_str(),
            _ => return None,
        }
    } else if matches_accessors!(dependency_accessors, ["target", _, _, _]) {
        match &dependency_accessors[2] {
            Accessor::Key(key) => key.as_str(),
            _ => return None,
        }
    } else {
        match &dependency_accessors[0] {
            Accessor::Key(key) => key.as_str(),
            _ => return None,
        }
    };

    let workspace_keys = [
        "workspace",
        sanitize_dependency_key(dependency_kind),
        dependency_key,
    ];
    let (_, workspace_dependency_value) = dig_keys(&workspace_document_tree, &workspace_keys)?;
    let Value::Table(workspace_dependency_table) = workspace_dependency_value else {
        return None;
    };

    let Value::String(path) = workspace_dependency_table.get("path")? else {
        return None;
    };

    let (resolved_cargo_toml_path, _, _) = find_cargo_toml(
        &workspace_cargo_toml_path,
        Path::new(path.value()),
        toml_version,
    )?;

    load_package_metadata(&resolved_cargo_toml_path, toml_version)
}

fn is_unsupported_remote_dependency(dependency_value: &Value) -> bool {
    let Value::Table(table) = dependency_value else {
        return false;
    };

    table.contains_key("git") || table.contains_key("registry")
}

fn load_package_metadata(
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<HoverMetadata> {
    let (_, document_tree) = load_cargo_toml(cargo_toml_path, toml_version)?;
    let package_name = match dig_keys(&document_tree, &["package", "name"]) {
        Some((_, Value::String(name))) => Some(name.value().to_string()),
        _ => None,
    };
    let description = match dig_keys(&document_tree, &["package", "description"]) {
        Some((_, Value::String(description))) => Some(description.value().to_string()),
        _ => None,
    };

    if package_name.is_none() && description.is_none() {
        return None;
    }

    Some(HoverMetadata {
        title: package_name,
        description,
    })
}

async fn fetch_crates_io_metadata(
    package_name: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<HoverMetadata>, tower_lsp::jsonrpc::Error> {
    let url = format!("https://crates.io/api/v1/crates/{package_name}");
    let Some(response) =
        fetch_cached_remote_json::<CratesIoCrateResponse>(&url, offline, cache_options).await
    else {
        return Ok(None);
    };

    if response.crate_info.name.is_none()
        && response.crate_info.description.is_none()
        && response.crate_info.max_version.is_none()
    {
        return Ok(None);
    }

    Ok(Some(HoverMetadata {
        title: response.crate_info.name,
        description: append_latest_version(
            response.crate_info.description,
            response.crate_info.max_version,
        ),
    }))
}

#[cfg(test)]
mod tests {
    use tombi_ast::AstNode;
    use tombi_document_tree::TryIntoDocumentTree;

    use super::*;

    #[test]
    fn parses_crates_io_metadata_response() {
        let response: CratesIoCrateResponse = serde_json::from_str(
            r#"{
                "crate": {
                    "name": "serde",
                    "description": "A generic serialization/deserialization framework",
                    "max_version": "1.0.228"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(response.crate_info.name.as_deref(), Some("serde"));
        assert_eq!(
            response.crate_info.description.as_deref(),
            Some("A generic serialization/deserialization framework")
        );
        assert_eq!(response.crate_info.max_version.as_deref(), Some("1.0.228"));
    }

    #[test]
    fn only_matches_exact_dependency_paths() {
        let dependency_accessors = [
            Accessor::Key("dependencies".into()),
            Accessor::Key("serde".into()),
        ];
        let nested_accessors = [
            Accessor::Key("dependencies".into()),
            Accessor::Key("serde".into()),
            Accessor::Key("version".into()),
        ];

        assert_eq!(
            get_dependency_accessors(&dependency_accessors),
            Some(&dependency_accessors[..])
        );
        assert_eq!(get_dependency_accessors(&nested_accessors), None);
    }

    #[test]
    fn hovering_dependency_value_does_not_count_as_hovering_key() {
        let source = "[dependencies]\nserde = \"1.0\"\n";
        let root = tombi_ast::Root::cast(tombi_parser::parse(source).into_syntax_node()).unwrap();
        let document_tree = root.try_into_document_tree(TomlVersion::V1_0_0).unwrap();
        let dependency_accessors = [
            Accessor::Key("dependencies".into()),
            Accessor::Key("serde".into()),
        ];

        let key_position = tombi_text::Position::default()
            + tombi_text::RelativePosition::of("[dependencies]\nse");
        let value_position = tombi_text::Position::default()
            + tombi_text::RelativePosition::of("[dependencies]\nserde = \"1");

        assert!(is_hovering_dependency_key(
            &document_tree,
            &dependency_accessors,
            key_position,
        ));
        assert!(!is_hovering_dependency_key(
            &document_tree,
            &dependency_accessors,
            value_position,
        ));
    }
}
