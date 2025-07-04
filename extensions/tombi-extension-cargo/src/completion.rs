use ahash::AHashMap;
use itertools::Itertools;
use serde::Deserialize;
use tombi_config::TomlVersion;
use tombi_extension::CompletionContent;
use tombi_extension::CompletionHint;
use tombi_extension::CompletionKind;
use tombi_future::Boxable;
use tombi_schema_store::dig_accessors;
use tombi_schema_store::matches_accessors;
use tombi_schema_store::Accessor;
use tombi_schema_store::HttpClient;
use tombi_version_sort::version_sort;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::find_path_crate_cargo_toml;
use crate::find_workspace_cargo_toml;
use crate::get_workspace_path;

#[derive(Debug, Deserialize)]
struct CratesIoVersionsResponse {
    versions: Vec<CratesIoVersion>,
}

#[derive(Debug, Deserialize)]
struct CratesIoVersion {
    num: String,
    features: AHashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct CratesIoCrateResponse {
    #[serde(default)]
    versions: Vec<CratesIoVersion>,
}

#[derive(Debug, Deserialize)]
struct CratesIoVersionDetailResponse {
    version: CratesIoVersion,
}

pub async fn completion(
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
    toml_version: TomlVersion,
    completion_hint: Option<CompletionHint>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if !text_document.uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }
    let cargo_toml_path = std::path::Path::new(text_document.uri.path());

    if let Some(Accessor::Key(first)) = accessors.first() {
        if first == "workspace" {
            completion_workspace(
                document_tree,
                cargo_toml_path,
                position,
                accessors,
                completion_hint,
                toml_version,
            )
            .await
        } else {
            completion_member(
                document_tree,
                cargo_toml_path,
                position,
                accessors,
                completion_hint,
                toml_version,
            )
            .await
        }
    } else {
        Ok(None)
    }
}

async fn completion_workspace(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    position: tombi_text::Position,
    accessors: &[Accessor],
    completion_hint: Option<CompletionHint>,
    toml_version: TomlVersion,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if matches_accessors!(accessors, ["workspace", "dependencies", _]) {
        if let Some(Accessor::Key(crate_name)) = accessors.last() {
            return complete_crate_version(
                crate_name.as_str(),
                document_tree,
                accessors,
                position,
                completion_hint,
            )
            .await;
        }
    } else if matches_accessors!(accessors, ["workspace", "dependencies", _, "version"]) {
        if let Some(Accessor::Key(crate_name)) = accessors.get(accessors.len() - 2) {
            return complete_crate_version(
                crate_name.as_str(),
                document_tree,
                accessors,
                position,
                completion_hint,
            )
            .await;
        }
    } else if matches_accessors!(accessors, ["workspace", "dependencies", _, "features"])
        | matches_accessors!(accessors, ["workspace", "dependencies", _, "features", _])
    {
        if let Some(Accessor::Key(crate_name)) = accessors.get(2) {
            if let Some((_, tombi_document_tree::Value::Incomplete { .. })) =
                dig_accessors(document_tree, accessors)
            {
                return Ok(None);
            }

            return complete_crate_feature(
                crate_name.as_str(),
                document_tree,
                cargo_toml_path,
                &accessors[..4],
                position,
                toml_version,
                accessors.get(4).and_then(|_| {
                    dig_accessors(document_tree, accessors).and_then(|(_, feature)| {
                        if let tombi_document_tree::Value::String(feature_string) = feature {
                            Some(feature_string)
                        } else {
                            None
                        }
                    })
                }),
            )
            .await;
        }
    }
    Ok(None)
}

async fn completion_member(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    position: tombi_text::Position,
    accessors: &[Accessor],
    completion_hint: Option<CompletionHint>,
    toml_version: TomlVersion,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if matches_accessors!(accessors, ["dependencies", _, "version"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "version"])
        || matches_accessors!(accessors, ["build-dependencies", _, "version"])
    {
        if let Some(Accessor::Key(c_name)) = accessors.get(accessors.len() - 2) {
            return complete_crate_version(
                c_name.as_str(),
                document_tree,
                accessors,
                position,
                completion_hint,
            )
            .await;
        }
    } else if matches_accessors!(accessors, ["dependencies", _])
        || matches_accessors!(accessors, ["dev-dependencies", _])
        || matches_accessors!(accessors, ["build-dependencies", _])
    {
        if let Some(Accessor::Key(c_name)) = accessors.last() {
            return complete_crate_version(
                c_name.as_str(),
                document_tree,
                accessors,
                position,
                completion_hint,
            )
            .await;
        }
    } else if (matches_accessors!(accessors, ["dependencies", _, "features", _])
        || matches_accessors!(accessors, ["dev-dependencies", _, "features", _])
        || matches_accessors!(accessors, ["build-dependencies", _, "features", _])
        || matches_accessors!(accessors, ["dependencies", _, "features"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "features"])
        || matches_accessors!(accessors, ["build-dependencies", _, "features"]))
    {
        if let Some(Accessor::Key(crate_name)) = accessors.get(1) {
            if let Some((_, tombi_document_tree::Value::Incomplete { .. })) =
                dig_accessors(document_tree, accessors)
            {
                return Ok(None);
            }

            return complete_crate_feature(
                crate_name.as_str(),
                document_tree,
                cargo_toml_path,
                &accessors[..3],
                position,
                toml_version,
                accessors.get(3).and_then(|_| {
                    dig_accessors(document_tree, accessors).and_then(|(_, feature)| {
                        if let tombi_document_tree::Value::String(feature_string) = feature {
                            Some(feature_string)
                        } else {
                            None
                        }
                    })
                }),
            )
            .await;
        }
    }
    Ok(None)
}

async fn complete_crate_version(
    crate_name: &str,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    position: tombi_text::Position,
    completion_hint: Option<CompletionHint>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    let version_value = match dig_accessors(document_tree, accessors) {
        Some((_, tombi_document_tree::Value::String(value_string))) => Some(value_string),
        Some((_, tombi_document_tree::Value::Incomplete { .. })) => None,
        _ => return Ok(None),
    };
    if let Some(versions) = fetch_crate_versions(crate_name).await {
        let items = versions
            .into_iter()
            .sorted_by(|a, b| tombi_version_sort::version_sort(a, b))
            .rev()
            .take(100)
            .enumerate()
            .map(|(i, ver)| CompletionContent {
                label: format!("\"{ver}\""),
                kind: CompletionKind::String,
                emoji_icon: Some('🦀'),
                priority: tombi_extension::CompletionContentPriority::Custom(format!(
                    "10__cargo_{i:>03}__",
                )),
                detail: Some("Crate version".to_string()),
                documentation: None,
                filter_text: None,
                schema_url: None,
                deprecated: None,
                edit: match version_value {
                    Some(value) => Some(tombi_extension::CompletionEdit {
                        text_edit: tower_lsp::lsp_types::CompletionTextEdit::Edit(
                            tower_lsp::lsp_types::TextEdit {
                                range: tombi_text::Range::at(position).into(),
                                new_text: format!("\"{ver}\""),
                            },
                        ),
                        insert_text_format: Some(
                            tower_lsp::lsp_types::InsertTextFormat::PLAIN_TEXT,
                        ),
                        additional_text_edits: Some(vec![tower_lsp::lsp_types::TextEdit {
                            range: value.range().into(),
                            new_text: "".to_string(),
                        }]),
                    }),
                    None => tombi_extension::CompletionEdit::new_literal(
                        &format!("\"{ver}\""),
                        position,
                        completion_hint,
                    ),
                },
                preselect: None,
            })
            .collect();
        Ok(Some(items))
    } else {
        Ok(None)
    }
}

fn complete_crate_feature<'a: 'b, 'b>(
    crate_name: &'a str,
    document_tree: &'a tombi_document_tree::DocumentTree,
    cargo_toml_path: &'a std::path::Path,
    features_accessors: &'a [Accessor],
    position: tombi_text::Position,
    toml_version: TomlVersion,
    editing_feature_string: Option<&'a tombi_document_tree::String>,
) -> tombi_future::BoxFuture<'b, Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error>>
{
    async move {
        // Check if this is a path dependency
        let features = if let Some((_, tombi_document_tree::Value::String(path_value))) =
            dig_accessors(
                document_tree,
                &features_accessors[..features_accessors.len() - 1]
                    .iter()
                    .chain(std::iter::once(&Accessor::Key("path".to_string())))
                    .cloned()
                    .collect_vec(),
            ) {
            // This is a path dependency - read features from local Cargo.toml
            fetch_local_crate_features(cargo_toml_path, path_value.value(), toml_version).await
        } else if let Some((_, tombi_document_tree::Value::String(value_string))) = dig_accessors(
            document_tree,
            &features_accessors[..features_accessors.len() - 1]
                .iter()
                .chain(std::iter::once(&Accessor::Key("version".to_string())))
                .cloned()
                .collect_vec(),
        ) {
            fetch_crate_features(crate_name, Some(value_string.value())).await
        } else if let Some((_, tombi_document_tree::Value::Boolean(boolean))) = dig_accessors(
            document_tree,
            &features_accessors[..features_accessors.len() - 1]
                .iter()
                .chain(std::iter::once(&Accessor::Key("workspace".to_string())))
                .cloned()
                .collect_vec(),
        ) {
            if boolean.value() {
                let Some((workspace_cargo_toml_path, workspace_document_tree)) =
                    find_workspace_cargo_toml(
                        cargo_toml_path,
                        get_workspace_path(document_tree),
                        toml_version,
                    )
                else {
                    return Ok(None);
                };
                return complete_crate_feature(
                    crate_name,
                    &workspace_document_tree,
                    &workspace_cargo_toml_path,
                    &[
                        Accessor::Key("workspace".to_string()),
                        Accessor::Key("dependencies".to_string()),
                        Accessor::Key(crate_name.to_string()),
                        Accessor::Key("features".to_string()),
                    ],
                    position,
                    toml_version,
                    editing_feature_string,
                )
                .await;
            } else {
                fetch_crate_features(crate_name, None).await
            }
        } else {
            fetch_crate_features(crate_name, None).await
        };

        let Some(features) = features else {
            return Ok(None);
        };

        let already_features: Vec<String> = match dig_accessors(document_tree, features_accessors) {
            Some((_, tombi_document_tree::Value::Array(array))) => array
                .values()
                .iter()
                .filter_map(|feature| {
                    if let tombi_document_tree::Value::String(feature_string) = feature {
                        Some(feature_string.value().to_string())
                    } else {
                        None
                    }
                })
                .collect(),
            _ => Vec::with_capacity(0),
        };

        let items = features
            .into_iter()
            .filter(|(feature, _)| !already_features.contains(feature))
            .sorted_by(|(a, _), (b, _)| version_sort(a, b))
            .enumerate()
            .map(|(i, (feature, feature_dependencies))| CompletionContent {
                label: format!("\"{}\"", feature),
                kind: CompletionKind::String,
                emoji_icon: Some('🦀'),
                priority: tombi_extension::CompletionContentPriority::Custom(format!(
                    "10__cargo_feature_{:>03}__",
                    if feature == "default" {
                        0 // default feature should be the first
                    } else if feature.starts_with('_') {
                        900 + i // features starting with `_` are considered private
                    } else {
                        i + 1
                    }
                )),
                detail: Some("Crate feature".to_string()),
                documentation: (!feature_dependencies.is_empty()).then(|| {
                    "Feature dependencies:\n".to_string()
                        + &feature_dependencies
                            .into_iter()
                            .map(|dep| format!("- `{dep}`"))
                            .collect_vec()
                            .join("\n")
                }),
                filter_text: None,
                schema_url: None,
                deprecated: None,
                edit: editing_feature_string.map(|value| tombi_extension::CompletionEdit {
                    text_edit: tower_lsp::lsp_types::CompletionTextEdit::Edit(
                        tower_lsp::lsp_types::TextEdit {
                            range: tombi_text::Range::at(position).into(),
                            new_text: format!("\"{feature}\""),
                        },
                    ),
                    insert_text_format: Some(tower_lsp::lsp_types::InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: Some(vec![tower_lsp::lsp_types::TextEdit {
                        range: value.range().into(),
                        new_text: "".to_string(),
                    }]),
                }),
                preselect: None,
            })
            .collect();
        Ok(Some(items))
    }
    .boxed()
}

/// Fetch crate version list from crates.io API
async fn fetch_crate_versions(crate_name: &str) -> Option<Vec<String>> {
    let url = format!("https://crates.io/api/v1/crates/{}/versions", crate_name);
    let client = HttpClient::new();
    let bytes = match client
        .get_bytes(&url)
        .await
        .map_err(|e| format!("http error: {e:?}"))
    {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to fetch crate versions from {url}: {e}");
            return None;
        }
    };

    let resp: CratesIoVersionsResponse = match serde_json::from_slice(&bytes) {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to parse crate versions response: {e}");
            return None;
        }
    };
    Some(resp.versions.into_iter().map(|v| v.num).collect())
}

/// Fetch crate features list from crates.io API
async fn fetch_crate_features(
    crate_name: &str,
    version: Option<&str>,
) -> Option<AHashMap<String, Vec<String>>> {
    let client = HttpClient::new();
    let version = if let Some(ver) = version {
        ver.to_string()
    } else {
        // fetch latest version
        let url = format!("https://crates.io/api/v1/crates/{crate_name}");
        let bytes = client.get_bytes(&url).await.ok()?;
        let resp: CratesIoCrateResponse = serde_json::from_slice(&bytes).ok()?;

        resp.versions.into_iter().next().map(|v| v.num)?
    };
    let url = format!("https://crates.io/api/v1/crates/{crate_name}/{version}");
    let bytes = client.get_bytes(&url).await.ok()?;
    let resp: CratesIoVersionDetailResponse = serde_json::from_slice(&bytes).ok()?;
    Some(resp.version.features)
}

/// Fetch crate features from local path Cargo.toml
async fn fetch_local_crate_features(
    cargo_toml_path: &std::path::Path,
    sub_crate_path: &str,
    toml_version: TomlVersion,
) -> Option<AHashMap<String, Vec<String>>> {
    // Get the directory of the current Cargo.toml file
    let (_, subcrate_document_tree) = find_path_crate_cargo_toml(
        cargo_toml_path,
        std::path::Path::new(sub_crate_path),
        toml_version,
    )?;

    // Extract features from [features] section
    if let Some((_, tombi_document_tree::Value::Table(features_table))) =
        tombi_document_tree::dig_keys(&subcrate_document_tree, &["features"])
    {
        let mut features = AHashMap::new();

        for (feature_name, feature_deps) in features_table.key_values() {
            let feature_name = feature_name.value().to_string();
            let deps = match feature_deps {
                tombi_document_tree::Value::Array(arr) => arr
                    .values()
                    .iter()
                    .filter_map(|v| {
                        if let tombi_document_tree::Value::String(s) = v {
                            Some(s.value().to_string())
                        } else {
                            None
                        }
                    })
                    .collect(),
                _ => Vec::new(),
            };
            features.insert(feature_name, deps);
        }

        return Some(features);
    }

    None
}
