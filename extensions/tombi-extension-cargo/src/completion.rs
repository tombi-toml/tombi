use itertools::Itertools;
use serde::Deserialize;
use tombi_config::TomlVersion;
use tombi_document_tree::{dig_accessors, dig_keys};
use tombi_extension::CommentContext;
use tombi_extension::CompletionContent;
use tombi_extension::CompletionContentPriority;
use tombi_extension::CompletionHint;
use tombi_extension::CompletionKind;
use tombi_extension::CompletionTextEdit;
use tombi_extension::TextEdit;
use tombi_extension::fetch_cached_remote_json;
use tombi_extension::{completion_directory_path, completion_file_path_from_uri};
use tombi_future::Boxable;
use tombi_hashmap::HashSet;
use tombi_schema_store::Accessor;
use tombi_schema_store::matches_accessors;
use tombi_version_sort::version_sort;
use tower_lsp::lsp_types::InsertTextFormat;

use crate::cargo_lock::{exact_crates_io_version, load_cached_cargo_lock};
use crate::{
    find_cargo_toml, find_workspace_cargo_toml, get_workspace_cargo_toml_path,
    is_any_dependency_path_accessor, is_dependency_accessor,
};

enum CargoCompletionFeature {
    DependencyVersion,
    DependencyFeature,
    Path,
}

#[derive(Debug, Deserialize)]
struct CratesIoVersionsResponse {
    versions: Vec<CratesIoVersion>,
}

#[derive(Debug, Deserialize)]
struct CratesIoVersion {
    num: String,
    features: tombi_hashmap::HashMap<String, Vec<String>>,
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
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
    toml_version: TomlVersion,
    completion_hint: Option<CompletionHint>,
    comment_context: Option<&CommentContext>,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }

    if comment_context.is_some() {
        return Ok(None);
    }

    if !features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.completion())
        .map(|completion| completion.enabled())
        .unwrap_or_default()
        .value()
    {
        return Ok(None);
    }

    if let Some(completions) = cargo_completion_enabled(features, CargoCompletionFeature::Path)
        .then(|| completion_cargo_file_path(text_document_uri, document_tree, position, accessors))
        .flatten()
    {
        return Ok(Some(completions));
    }

    let cargo_toml_path = std::path::Path::new(text_document_uri.path());

    if let Some(Accessor::Key(first)) = accessors.first() {
        if first == "workspace" {
            completion_workspace(
                document_tree,
                cargo_toml_path,
                position,
                accessors,
                completion_hint,
                toml_version,
                offline,
                cache_options,
                features,
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
                offline,
                cache_options,
                features,
            )
            .await
        }
    } else {
        Ok(None)
    }
}

/// Tries directory-only completion, then .rs path completion, then any-file path completion.
fn completion_cargo_file_path(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
) -> Option<Vec<CompletionContent>> {
    if matches_accessors!(accessors, ["workspace", "members", _])
        || matches_accessors!(accessors, ["workspace", "exclude", _])
        || matches_accessors!(accessors, ["workspace", "default-members", _])
    {
        return completion_directory_path(text_document_uri, document_tree, position, accessors);
    }

    if (matches_accessors!(accessors, ["package", "build"])
        || matches_accessors!(accessors, ["project", "build"])
        || matches_accessors!(accessors, ["lib", "path"])
        || matches_accessors!(accessors, ["bin", _, "path"])
        || matches_accessors!(accessors, ["example", _, "path"])
        || matches_accessors!(accessors, ["test", _, "path"])
        || matches_accessors!(accessors, ["bench", _, "path"]))
        && let Some(completions) = completion_file_path_from_uri(
            text_document_uri,
            document_tree,
            position,
            accessors,
            Some(&["rs"]),
        )
    {
        return Some(completions);
    }

    if (is_any_dependency_path_accessor(accessors)
        || matches_accessors!(accessors, ["patch", _, _, "path"])
        || matches_accessors!(accessors, ["replace", _, "path"])
        || matches_accessors!(accessors, ["package", "readme"])
        || matches_accessors!(accessors, ["project", "readme"])
        || matches_accessors!(accessors, ["workspace", "package", "readme"])
        || matches_accessors!(accessors, ["package", "license-file"])
        || matches_accessors!(accessors, ["project", "license-file"])
        || matches_accessors!(accessors, ["workspace", "package", "license-file"])
        || matches_accessors!(accessors, ["package", "workspace"])
        || matches_accessors!(accessors, ["project", "workspace"])
        || matches_accessors!(accessors, ["package", "include", _])
        || matches_accessors!(accessors, ["project", "include", _])
        || matches_accessors!(accessors, ["workspace", "package", "include", _])
        || matches_accessors!(accessors, ["package", "exclude", _])
        || matches_accessors!(accessors, ["project", "exclude", _])
        || matches_accessors!(accessors, ["workspace", "package", "exclude", _])
        || matches_accessors!(accessors, ["package", "metadata", "playdate", "image-path"])
        || matches_accessors!(accessors, ["project", "metadata", "playdate", "image-path"])
        || matches_accessors!(
            accessors,
            ["package", "metadata", "playdate", "launch-sound-path"]
        )
        || matches_accessors!(
            accessors,
            ["project", "metadata", "playdate", "launch-sound-path"]
        )
        || matches_accessors!(accessors, ["package", "metadata", "playdate", "assets", _])
        || matches_accessors!(accessors, ["project", "metadata", "playdate", "assets", _])
        || matches_accessors!(
            accessors,
            ["package", "metadata", "playdate", "dev-assets", _]
        )
        || matches_accessors!(
            accessors,
            ["project", "metadata", "playdate", "dev-assets", _]
        ))
        && let Some(completions) = completion_file_path_from_uri(
            text_document_uri,
            document_tree,
            position,
            accessors,
            Some(&[]),
        )
    {
        return Some(completions);
    }

    None
}

async fn completion_workspace(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    position: tombi_text::Position,
    accessors: &[Accessor],
    completion_hint: Option<CompletionHint>,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if matches_accessors!(accessors, ["workspace", "dependencies", _]) {
        if !cargo_completion_enabled(features, CargoCompletionFeature::DependencyVersion) {
            return Ok(None);
        }
        if let Some(Accessor::Key(crate_name)) = accessors.last() {
            return complete_crate_version(
                crate_name.as_str(),
                document_tree,
                accessors,
                position,
                completion_hint,
                offline,
                cache_options,
            )
            .await;
        }
    } else if matches_accessors!(accessors, ["workspace", "dependencies", _, "version"]) {
        if !cargo_completion_enabled(features, CargoCompletionFeature::DependencyVersion) {
            return Ok(None);
        }
        if let Some(Accessor::Key(crate_name)) = accessors.get(accessors.len() - 2) {
            return complete_crate_version(
                crate_name.as_str(),
                document_tree,
                accessors,
                position,
                completion_hint,
                offline,
                cache_options,
            )
            .await;
        }
    } else if matches_accessors!(accessors, ["workspace", "dependencies", _, "features"])
        | matches_accessors!(accessors, ["workspace", "dependencies", _, "features", _])
        && let Some(Accessor::Key(crate_name)) = accessors.get(2)
    {
        if !cargo_completion_enabled(features, CargoCompletionFeature::DependencyFeature) {
            return Ok(None);
        }
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
            offline,
            cache_options,
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
    Ok(None)
}

async fn completion_member(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    position: tombi_text::Position,
    accessors: &[Accessor],
    completion_hint: Option<CompletionHint>,
    toml_version: TomlVersion,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if let Some(completions) = complete_workspace_dependency_inheritance(
        document_tree,
        cargo_toml_path,
        position,
        accessors,
        completion_hint,
        toml_version,
    ) {
        return Ok(Some(completions));
    }

    if matches_accessors!(accessors, ["dependencies", _, "version"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "version"])
        || matches_accessors!(accessors, ["build-dependencies", _, "version"])
        || matches_accessors!(accessors, ["target", _, "dependencies", _, "version"])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, "version"])
        || matches_accessors!(accessors, ["target", _, "build-dependencies", _, "version"])
    {
        if !cargo_completion_enabled(features, CargoCompletionFeature::DependencyVersion) {
            return Ok(None);
        }
        if let Some(Accessor::Key(c_name)) = accessors.get(accessors.len() - 2) {
            return complete_crate_version(
                c_name.as_str(),
                document_tree,
                accessors,
                position,
                completion_hint,
                offline,
                cache_options,
            )
            .await;
        }
    } else if is_dependency_accessor(accessors) {
        if !cargo_completion_enabled(features, CargoCompletionFeature::DependencyVersion) {
            return Ok(None);
        }
        if let Some(Accessor::Key(c_name)) = accessors.last() {
            return complete_crate_version(
                c_name.as_str(),
                document_tree,
                accessors,
                position,
                completion_hint,
                offline,
                cache_options,
            )
            .await;
        }
    } else if (matches_accessors!(accessors, ["dependencies", _, "features", _])
        || matches_accessors!(accessors, ["dev-dependencies", _, "features", _])
        || matches_accessors!(accessors, ["build-dependencies", _, "features", _])
        || matches_accessors!(accessors, ["dependencies", _, "features"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "features"])
        || matches_accessors!(accessors, ["build-dependencies", _, "features"])
        || matches_accessors!(accessors, ["target", _, "dependencies", _, "features", _])
        || matches_accessors!(
            accessors,
            ["target", _, "dev-dependencies", _, "features", _]
        )
        || matches_accessors!(
            accessors,
            ["target", _, "build-dependencies", _, "features", _]
        )
        || matches_accessors!(accessors, ["target", _, "dependencies", _, "features"])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, "features"])
        || matches_accessors!(
            accessors,
            ["target", _, "build-dependencies", _, "features"]
        ))
    {
        if !cargo_completion_enabled(features, CargoCompletionFeature::DependencyFeature) {
            return Ok(None);
        }
        let is_target_dependency = accessors.first().map(|a| a.as_key()) == Some(Some("target"));
        let offset = if is_target_dependency { 2 } else { 0 };

        if let Some(Accessor::Key(crate_name)) = accessors.get(1 + offset) {
            if let Some((_, tombi_document_tree::Value::Incomplete { .. })) =
                dig_accessors(document_tree, accessors)
            {
                return Ok(None);
            }

            return complete_crate_feature(
                crate_name.as_str(),
                document_tree,
                cargo_toml_path,
                &accessors[..3 + offset],
                position,
                toml_version,
                offline,
                cache_options,
                accessors.get(3 + offset).and_then(|_| {
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

fn cargo_completion_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
    feature: CargoCompletionFeature,
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.completion())
        .and_then(|completion| match feature {
            CargoCompletionFeature::DependencyVersion => completion.dependency_version(),
            CargoCompletionFeature::DependencyFeature => completion.dependency_feature(),
            CargoCompletionFeature::Path => completion.path(),
        })
        .and_then(|feature| feature.enabled)
        .unwrap_or_default()
        .value()
}

fn complete_workspace_dependency_inheritance(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    position: tombi_text::Position,
    accessors: &[Accessor],
    completion_hint: Option<CompletionHint>,
    toml_version: TomlVersion,
) -> Option<Vec<CompletionContent>> {
    if completion_hint.is_some() {
        return None;
    }

    let (dependency_table_accessors, dependency_name) = member_dependency_accessors(accessors)?;

    let Some((_, tombi_document_tree::Value::Table(current_dependency_table))) =
        dig_accessors(document_tree, dependency_table_accessors)
    else {
        return None;
    };

    let completion_range = if let Some(Accessor::Key(dependency_name)) = dependency_name {
        let Some((Accessor::Key(_), tombi_document_tree::Value::Incomplete { .. })) =
            dig_accessors(document_tree, accessors)
        else {
            return None;
        };

        let (current_dependency_key, _) =
            current_dependency_table.get_key_value(dependency_name.as_str())?;

        current_dependency_key.range()
    } else {
        tombi_text::Range::at(position)
    };

    let (_, _, workspace_document_tree) = find_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_cargo_toml_path(document_tree),
        toml_version,
    )?;

    let Some((_, tombi_document_tree::Value::Table(workspace_dependencies))) =
        dig_keys(&workspace_document_tree, &["workspace", "dependencies"])
    else {
        return None;
    };

    let existing_dependency_names = current_dependency_table
        .keys()
        .map(|key| key.value.clone())
        .filter(|key| Some(key.as_str()) != dependency_name.and_then(|key| key.as_key()))
        .collect::<HashSet<_>>();

    let dependency_prefix = dependency_name
        .and_then(|key| key.as_key())
        .unwrap_or_default();
    let completions = workspace_dependencies
        .keys()
        .filter(|key| key.value.starts_with(dependency_prefix))
        .filter(|key| !existing_dependency_names.contains(&key.value))
        .enumerate()
        .map(|(index, key)| CompletionContent {
            label: key.value.clone(),
            kind: CompletionKind::Key,
            emoji_icon: Some('🦀'),
            priority: CompletionContentPriority::Custom(format!(
                "10__cargo_workspace_dependency_{index:>03}__",
            )),
            detail: Some("Workspace dependency".to_string()),
            documentation: Some(
                "Inherit this dependency from `[workspace.dependencies]`.".to_string(),
            ),
            filter_text: None,
            schema_uri: None,
            deprecated: None,
            edit: Some(tombi_extension::CompletionEdit {
                text_edit: CompletionTextEdit::Edit(TextEdit {
                    range: completion_range,
                    new_text: format!("{}.workspace = true", key.value),
                }),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                additional_text_edits: None,
            }),
            preselect: None,
            in_comment: false,
        })
        .collect::<Vec<_>>();

    if completions.is_empty() {
        None
    } else {
        Some(completions)
    }
}

fn member_dependency_accessors(accessors: &[Accessor]) -> Option<(&[Accessor], Option<&Accessor>)> {
    if matches_accessors!(accessors, ["dependencies"])
        || matches_accessors!(accessors, ["dev-dependencies"])
        || matches_accessors!(accessors, ["build-dependencies"])
    {
        Some((accessors, None))
    } else if matches_accessors!(accessors, ["dependencies", _])
        || matches_accessors!(accessors, ["dev-dependencies", _])
        || matches_accessors!(accessors, ["build-dependencies", _])
    {
        Some((&accessors[..1], accessors.get(1)))
    } else if matches_accessors!(accessors, ["target", _, "dependencies"])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies"])
        || matches_accessors!(accessors, ["target", _, "build-dependencies"])
    {
        Some((accessors, None))
    } else if matches_accessors!(accessors, ["target", _, "dependencies", _])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _])
        || matches_accessors!(accessors, ["target", _, "build-dependencies", _])
    {
        Some((&accessors[..3], accessors.get(3)))
    } else {
        None
    }
}

async fn complete_crate_version(
    crate_name: &str,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
    position: tombi_text::Position,
    completion_hint: Option<CompletionHint>,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    let version_value = match dig_accessors(document_tree, accessors) {
        Some((_, value))
            if matches!(
                value,
                tombi_document_tree::Value::String(_)
                    | tombi_document_tree::Value::Incomplete { .. }
            ) =>
        {
            value
        }
        _ => return Ok(None),
    };

    if let Some(versions) = fetch_crate_versions(crate_name, offline, cache_options).await {
        let items = versions
            .into_iter()
            .sorted_by(|a, b| tombi_version_sort::version_sort(a, b))
            .rev()
            .take(100)
            .enumerate()
            .map(|(i, version_str)| CompletionContent {
                label: format!("\"{version_str}\""),
                kind: CompletionKind::Enum,
                emoji_icon: Some('🦀'),
                priority: tombi_extension::CompletionContentPriority::Custom(format!(
                    "10__cargo_{i:>03}__",
                )),
                detail: Some("Crate version".to_string()),
                documentation: None,
                filter_text: None,
                schema_uri: None,
                deprecated: None,
                edit: match version_value {
                    tombi_document_tree::Value::String(value_string) => {
                        tombi_extension::CompletionEdit::new_string_literal_while_editing(
                            &format!("\"{version_str}\""),
                            value_string.range(),
                        )
                    }
                    tombi_document_tree::Value::Incomplete { .. } => {
                        Some(tombi_extension::CompletionEdit {
                            text_edit: CompletionTextEdit::Edit(tombi_extension::TextEdit {
                                range: tombi_text::Range::at(position),
                                new_text: format!(" = \"{version_str}\""),
                            }),
                            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                            additional_text_edits: match completion_hint {
                                Some(
                                    CompletionHint::DotTrigger { range, .. }
                                    | CompletionHint::EqualTrigger { range, .. },
                                ) => Some(vec![TextEdit {
                                    range,
                                    new_text: "".to_string(),
                                }]),
                                _ => None,
                            },
                        })
                    }
                    _ => None,
                },
                preselect: None,
                in_comment: false,
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
    offline: bool,
    cache_options: Option<&'a tombi_cache::Options>,
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
            let resolved_version = resolve_registry_dependency_version(
                cargo_toml_path,
                crate_name,
                value_string.value(),
                toml_version,
            )
            .await;
            fetch_crate_features(
                crate_name,
                resolved_version.as_deref(),
                offline,
                cache_options,
            )
            .await
        } else if let Some((_, tombi_document_tree::Value::Boolean(boolean))) = dig_accessors(
            document_tree,
            &features_accessors[..features_accessors.len() - 1]
                .iter()
                .chain(std::iter::once(&Accessor::Key("workspace".to_string())))
                .cloned()
                .collect_vec(),
        ) {
            if boolean.value() {
                let Some((workspace_cargo_toml_path, _, workspace_document_tree)) =
                    find_workspace_cargo_toml(
                        cargo_toml_path,
                        get_workspace_cargo_toml_path(document_tree),
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
                    offline,
                    cache_options,
                    editing_feature_string,
                )
                .await;
            } else {
                fetch_crate_features(crate_name, None, offline, cache_options).await
            }
        } else {
            fetch_crate_features(crate_name, None, offline, cache_options).await
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
                label: format!("\"{feature}\""),
                kind: CompletionKind::Enum,
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
                schema_uri: None,
                deprecated: None,
                edit: editing_feature_string.map(|value| tombi_extension::CompletionEdit {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        range: tombi_text::Range::at(position),
                        new_text: format!("\"{feature}\""),
                    }),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: Some(vec![TextEdit {
                        range: value.range(),
                        new_text: "".to_string(),
                    }]),
                }),
                preselect: None,
                in_comment: false,
            })
            .collect();
        Ok(Some(items))
    }
    .boxed()
}

/// Fetch crate version list from crates.io API
async fn fetch_crate_versions(
    crate_name: &str,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<Vec<String>> {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}/versions");
    let resp =
        fetch_cached_remote_json::<CratesIoVersionsResponse>(&url, offline, cache_options).await?;
    Some(resp.versions.into_iter().map(|v| v.num).collect())
}

/// Fetch crate features list from crates.io API
async fn fetch_crate_features(
    crate_name: &str,
    version: Option<&str>,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
) -> Option<tombi_hashmap::HashMap<String, Vec<String>>> {
    if let Some(version) = version {
        let url = format!("https://crates.io/api/v1/crates/{crate_name}/{version}");
        let resp =
            fetch_cached_remote_json::<CratesIoVersionDetailResponse>(&url, offline, cache_options)
                .await?;
        Some(resp.version.features)
    } else {
        // The crate overview response already includes features for each version,
        // so we can read them directly from the latest version without a second fetch.
        let url = format!("https://crates.io/api/v1/crates/{crate_name}");
        let resp =
            fetch_cached_remote_json::<CratesIoCrateResponse>(&url, offline, cache_options).await?;
        resp.versions.into_iter().next().map(|v| v.features)
    }
}

async fn resolve_registry_dependency_version(
    cargo_toml_path: &std::path::Path,
    crate_name: &str,
    version_requirement: &str,
    toml_version: TomlVersion,
) -> Option<String> {
    if let Some(lock) = load_cached_cargo_lock(cargo_toml_path, toml_version).await {
        return lock.resolve_dependency_version(crate_name, version_requirement);
    }

    exact_crates_io_version(version_requirement)
}

/// Fetch crate features from local path Cargo.toml
async fn fetch_local_crate_features(
    cargo_toml_path: &std::path::Path,
    sub_crate_path: &str,
    toml_version: TomlVersion,
) -> Option<tombi_hashmap::HashMap<String, Vec<String>>> {
    // Get the directory of the current Cargo.toml file
    let (_, _, subcrate_document_tree) = find_cargo_toml(
        cargo_toml_path,
        std::path::Path::new(sub_crate_path),
        toml_version,
    )?;

    // Extract features from [features] section
    if let Some((_, tombi_document_tree::Value::Table(features_table))) =
        tombi_document_tree::dig_keys(&subcrate_document_tree, &["features"])
    {
        let features = features_table
            .key_values()
            .iter()
            .map(|(feature_name, feature_deps)| {
                let deps = match feature_deps {
                    tombi_document_tree::Value::Array(arr) => arr
                        .values()
                        .iter()
                        .filter_map(|value| {
                            if let tombi_document_tree::Value::String(string) = value {
                                Some(string.value().to_string())
                            } else {
                                None
                            }
                        })
                        .collect(),
                    _ => Vec::with_capacity(0),
                };
                (feature_name.value.clone(), deps)
            })
            .collect::<tombi_hashmap::HashMap<_, _>>();

        if !features.is_empty() {
            return Some(features);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::OsString,
        fs,
        str::FromStr,
        sync::{LazyLock, Mutex, MutexGuard},
        time::Duration,
    };

    use super::*;

    static CACHE_ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    struct TestCacheHome {
        _guard: MutexGuard<'static, ()>,
        previous_tombi: Option<OsString>,
        previous_xdg: Option<OsString>,
        _temp_dir: tempfile::TempDir,
    }

    impl TestCacheHome {
        fn new() -> Self {
            let guard = CACHE_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
            let temp_dir = tempfile::tempdir().unwrap();
            let previous_tombi = std::env::var_os("TOMBI_CACHE_HOME");
            let previous_xdg = std::env::var_os("XDG_CACHE_HOME");
            // SAFETY: Tests serialize access with a process-wide mutex so env mutation
            // remains scoped to one test at a time.
            unsafe {
                std::env::remove_var("TOMBI_CACHE_HOME");
                std::env::set_var("XDG_CACHE_HOME", temp_dir.path());
            }
            Self {
                _guard: guard,
                previous_tombi,
                previous_xdg,
                _temp_dir: temp_dir,
            }
        }
    }

    impl Drop for TestCacheHome {
        fn drop(&mut self) {
            // SAFETY: Tests serialize access with a process-wide mutex so env mutation
            // remains scoped to one test at a time.
            unsafe {
                if let Some(previous) = &self.previous_tombi {
                    std::env::set_var("TOMBI_CACHE_HOME", previous);
                } else {
                    std::env::remove_var("TOMBI_CACHE_HOME");
                }

                if let Some(previous) = &self.previous_xdg {
                    std::env::set_var("XDG_CACHE_HOME", previous);
                } else {
                    std::env::remove_var("XDG_CACHE_HOME");
                }
            }
        }
    }

    fn cache_options() -> tombi_cache::Options {
        tombi_cache::Options {
            no_cache: None,
            cache_ttl: Some(Duration::from_secs(60)),
        }
    }

    fn unique_crate_name(suffix: &str) -> String {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("tombi-cache-test-{suffix}-{unique}")
    }

    async fn cached_remote_json_file_path(url: &str) -> std::path::PathBuf {
        let uri = tombi_uri::Uri::from_str(url).unwrap();
        tombi_cache::get_cache_file_path(&uri).await.unwrap()
    }

    async fn write_cached_response(url: &str, body: &str) {
        let cache_path = cached_remote_json_file_path(url).await;
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&cache_path, body).unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_crate_versions_uses_cached_response_while_offline() {
        let _cache_home = TestCacheHome::new();
        let crate_name = unique_crate_name("versions");
        let url = format!("https://crates.io/api/v1/crates/{crate_name}/versions");
        write_cached_response(
            &url,
            r#"{"versions":[{"num":"2.0.0","features":{}},{"num":"1.0.0","features":{}}]}"#,
        )
        .await;

        let versions = fetch_crate_versions(&crate_name, true, Some(&cache_options())).await;

        assert_eq!(
            versions,
            Some(vec!["2.0.0".to_string(), "1.0.0".to_string()])
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_crate_features_uses_cached_version_detail_while_offline() {
        let _cache_home = TestCacheHome::new();
        let crate_name = unique_crate_name("features-version");
        let version = "1.2.3";
        let url = format!("https://crates.io/api/v1/crates/{crate_name}/{version}");
        write_cached_response(
            &url,
            r#"{"version":{"num":"1.2.3","features":{"derive":[],"std":["dep:std"]}}}"#,
        )
        .await;

        let features =
            fetch_crate_features(&crate_name, Some(version), true, Some(&cache_options())).await;

        assert_eq!(
            features,
            Some(
                [
                    ("derive".to_string(), Vec::new()),
                    ("std".to_string(), vec!["dep:std".to_string()]),
                ]
                .into_iter()
                .collect()
            )
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_crate_features_uses_cached_latest_version_lookup_while_offline() {
        let _cache_home = TestCacheHome::new();
        let crate_name = unique_crate_name("features-latest");
        let url = format!("https://crates.io/api/v1/crates/{crate_name}");
        // The crate overview response includes features for each version,
        // so no second fetch for the version detail is needed.
        write_cached_response(
            &url,
            r#"{"versions":[{"num":"9.9.9","features":{"full":["derive"]}}]}"#,
        )
        .await;

        let features = fetch_crate_features(&crate_name, None, true, Some(&cache_options())).await;

        assert_eq!(
            features,
            Some(
                [("full".to_string(), vec!["derive".to_string()])]
                    .into_iter()
                    .collect()
            )
        );
    }

    #[test]
    fn exact_crates_io_version_accepts_plain_and_pinned_versions() {
        assert_eq!(exact_crates_io_version("1.2.3"), Some("1.2.3".to_string()));
        assert_eq!(exact_crates_io_version("=1.2.3"), Some("1.2.3".to_string()));
        assert_eq!(
            exact_crates_io_version("=0.9.12+spec-1.1.0"),
            Some("0.9.12+spec-1.1.0".to_string())
        );
        assert_eq!(exact_crates_io_version("^1.2"), None);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn resolve_registry_dependency_version_uses_lockfile_for_version_requirements() {
        let _cache_home = TestCacheHome::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let cargo_lock_path = temp_dir.path().join("Cargo.lock");
        fs::write(
            &cargo_toml_path,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            &cargo_lock_path,
            r#"
[[package]]
name = "demo"
version = "0.1.0"
dependencies = ["criterion 0.5.1"]

[[package]]
name = "criterion"
version = "0.5.1"
"#,
        )
        .unwrap();

        assert_eq!(
            resolve_registry_dependency_version(
                &cargo_toml_path,
                "criterion",
                "0.5",
                TomlVersion::default(),
            )
            .await,
            Some("0.5.1".to_string())
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn resolve_registry_dependency_version_prefers_lockfile_version_with_source_suffix() {
        let _cache_home = TestCacheHome::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let cargo_lock_path = temp_dir.path().join("Cargo.lock");
        fs::write(
            &cargo_toml_path,
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(
            &cargo_lock_path,
            r#"
version = 4

[[package]]
name = "demo"
version = "0.1.0"
dependencies = [
    "toml 0.9.12+spec-1.1.0 (registry+https://github.com/rust-lang/crates.io-index)",
]

[[package]]
name = "toml"
version = "0.9.12+spec-1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#,
        )
        .unwrap();

        assert_eq!(
            resolve_registry_dependency_version(
                &cargo_toml_path,
                "toml",
                "=0.9.10",
                TomlVersion::default(),
            )
            .await,
            Some("0.9.12+spec-1.1.0".to_string())
        );
    }
}
