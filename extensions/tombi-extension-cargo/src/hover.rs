use std::path::Path;

use serde::Deserialize;
use tombi_config::TomlVersion;
use tombi_document_tree::{Value, dig_accessors, dig_keys};
use tombi_extension::{HoverMetadata, fetch_cached_remote_json};
use tombi_schema_store::{Accessor, matches_accessors};

use crate::{
    find_path_crate_cargo_toml, find_workspace_cargo_toml, get_workspace_path, load_cargo_toml,
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
}

pub async fn hover(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
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

    let Some(dependency_accessors) = get_dependency_accessors(accessors) else {
        return Ok(None);
    };

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

fn get_dependency_accessors(accessors: &[Accessor]) -> Option<&[Accessor]> {
    if matches_accessors!(accessors, ["workspace", "dependencies", _])
        || matches_accessors!(accessors, ["workspace", "dependencies", _, _])
    {
        Some(&accessors[..3])
    } else if matches_accessors!(accessors, ["dependencies", _])
        || matches_accessors!(accessors, ["dependencies", _, _])
        || matches_accessors!(accessors, ["dev-dependencies", _])
        || matches_accessors!(accessors, ["dev-dependencies", _, _])
        || matches_accessors!(accessors, ["build-dependencies", _])
        || matches_accessors!(accessors, ["build-dependencies", _, _])
    {
        Some(&accessors[..2])
    } else if matches_accessors!(accessors, ["target", _, "dependencies", _])
        || matches_accessors!(accessors, ["target", _, "dependencies", _, _])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, _])
        || matches_accessors!(accessors, ["target", _, "build-dependencies", _])
        || matches_accessors!(accessors, ["target", _, "build-dependencies", _, _])
    {
        Some(&accessors[..4])
    } else {
        None
    }
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
                find_path_crate_cargo_toml(cargo_toml_path, Path::new(path.value()), toml_version)
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
        get_workspace_path(document_tree),
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

    let (resolved_cargo_toml_path, _, _) = find_path_crate_cargo_toml(
        &workspace_cargo_toml_path,
        Path::new(path.value()),
        toml_version,
    )?;

    load_package_metadata(&resolved_cargo_toml_path, toml_version)
}

fn dependency_package_name<'a>(dependency_key: &'a str, dependency_value: &'a Value) -> &'a str {
    match dependency_value {
        Value::Table(table) => match table.get("package") {
            Some(Value::String(package)) => package.value(),
            _ => dependency_key,
        },
        _ => dependency_key,
    }
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

    if response.crate_info.name.is_none() && response.crate_info.description.is_none() {
        return Ok(None);
    }

    Ok(Some(HoverMetadata {
        title: response.crate_info.name,
        description: response.crate_info.description,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_crates_io_metadata_response() {
        let response: CratesIoCrateResponse = serde_json::from_str(
            r#"{
                "crate": {
                    "name": "serde",
                    "description": "A generic serialization/deserialization framework"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(response.crate_info.name.as_deref(), Some("serde"));
        assert_eq!(
            response.crate_info.description.as_deref(),
            Some("A generic serialization/deserialization framework")
        );
    }
}
