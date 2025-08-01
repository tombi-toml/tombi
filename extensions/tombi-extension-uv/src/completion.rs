use std::str::FromStr;

use ahash::AHashMap;
use itertools::Itertools;
use pep508_rs::{Requirement, VerbatimUrl, VersionOrUrl};
use serde::Deserialize;
use tombi_config::TomlVersion;
use tombi_extension::CompletionContent;
use tombi_extension::CompletionHint;
use tombi_extension::CompletionKind;
use tombi_future::BoxFuture;
use tombi_future::Boxable;
use tombi_schema_store::dig_accessors;
use tombi_schema_store::matches_accessors;
use tombi_schema_store::Accessor;
use tombi_schema_store::HttpClient;
use tombi_version_sort::version_sort;
use tower_lsp::lsp_types::TextDocumentIdentifier;

#[derive(Debug, Deserialize)]
struct PyPiProjectResponse {
    info: PyPiProjectInfo,
    releases: AHashMap<String, Vec<PyPiReleaseInfo>>,
}

#[derive(Debug, Deserialize)]
struct PyPiProjectInfo {
    name: String,
    version: String,
    #[serde(default)]
    requires_dist: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PyPiReleaseInfo {
    #[serde(default)]
    yanked: bool,
}

#[derive(Debug, Deserialize)]
struct PyPiVersionResponse {
    info: PyPiVersionInfo,
}

#[derive(Debug, Deserialize)]
struct PyPiVersionInfo {
    name: String,
    version: String,
    #[serde(default)]
    requires_dist: Option<Vec<String>>,
    #[serde(default)]
    provides_extra: Option<Vec<String>>,
}

pub async fn completion(
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
    toml_version: TomlVersion,
    completion_hint: Option<CompletionHint>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if !text_document.uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    // Handle [project] dependencies
    if matches_accessors!(accessors, ["project", "dependencies", _]) {
        if let Some((_, tombi_document_tree::Value::String(dep_spec))) =
            dig_accessors(document_tree, accessors)
        {
            return complete_package_spec(dep_spec.value(), dep_spec, position, completion_hint)
                .await;
        }
    }

    // Handle [project] optional-dependencies
    if matches_accessors!(accessors, ["project", "optional-dependencies", _, _]) {
        if let Some((_, tombi_document_tree::Value::String(dep_spec))) =
            dig_accessors(document_tree, accessors)
        {
            return complete_package_spec(dep_spec.value(), dep_spec, position, completion_hint)
                .await;
        }
    }

    // Handle [tool.uv] dependencies
    if matches_accessors!(accessors, ["tool", "uv", "dependencies", _]) {
        if let Some((_, tombi_document_tree::Value::String(dep_spec))) =
            dig_accessors(document_tree, accessors)
        {
            return complete_package_spec(dep_spec.value(), dep_spec, position, completion_hint)
                .await;
        }
    }

    // Handle [dependency-groups]
    if matches_accessors!(accessors, ["dependency-groups", _, _]) {
        if let Some((_, tombi_document_tree::Value::String(dep_spec))) =
            dig_accessors(document_tree, accessors)
        {
            return complete_package_spec(dep_spec.value(), dep_spec, position, completion_hint)
                .await;
        }
    }

    Ok(None)
}

#[derive(Debug)]
enum CompletionContext {
    PackageName,
    AfterPackageName(String),
    FeatureName(String),
    VersionOperator(String),
    VersionNumber(String, String), // package_name, operator
}

fn analyze_completion_context(dep_spec: &str, cursor_pos: usize) -> CompletionContext {
    tracing::info!(
        "analyze_completion_context: dep_spec={:?}, cursor_pos={}",
        dep_spec,
        cursor_pos
    );

    // Try to parse as a complete requirement first
    if let Ok(requirement) = Requirement::<VerbatimUrl>::from_str(dep_spec) {
        let package_name = requirement.name.to_string();
        tracing::info!(
            "Parsed requirement successfully: package_name={}",
            package_name
        );

        // Check if we're in the extras/features section
        if dep_spec.contains('[') && dep_spec.contains(']') {
            if let Some(start) = dep_spec.find('[') {
                if let Some(end) = dep_spec.find(']') {
                    if cursor_pos > start && cursor_pos <= end {
                        tracing::info!("Context: In features section");
                        return CompletionContext::FeatureName(package_name);
                    }
                }
            }
        }

        // Check if we're after version operator
        if let Some(VersionOrUrl::VersionSpecifier(_)) = requirement.version_or_url {
            if let Some(op_pos) = dep_spec.rfind(|c: char| matches!(c, '=' | '>' | '<' | '!' | '~'))
            {
                if cursor_pos > op_pos {
                    let operator = &dep_spec[op_pos..op_pos + 2.min(dep_spec.len() - op_pos)];
                    tracing::info!("Context: After version operator={}", operator);
                    return CompletionContext::VersionNumber(package_name, operator.to_string());
                }
            }
        }

        tracing::info!("Context: After package name");
        return CompletionContext::AfterPackageName(package_name);
    } else {
        tracing::info!("Failed to parse as complete requirement");
    }

    // Handle incomplete specs
    if dep_spec.is_empty() {
        return CompletionContext::PackageName;
    }

    // Check for features syntax
    if let Some(bracket_pos) = dep_spec.find('[') {
        let package_name = dep_spec[..bracket_pos].trim();
        if cursor_pos > bracket_pos {
            return CompletionContext::FeatureName(package_name.to_string());
        }
    }

    // Check for version operators
    if let Some(op_idx) = dep_spec.find(|c: char| matches!(c, '=' | '>' | '<' | '!' | '~')) {
        let package_name = dep_spec[..op_idx].trim();
        tracing::info!(
            "Found operator at index {}, package_name={}",
            op_idx,
            package_name
        );

        if cursor_pos == op_idx + 1 {
            tracing::info!("Context: At version operator position");
            return CompletionContext::VersionOperator(package_name.to_string());
        } else if cursor_pos > op_idx {
            let operator = &dep_spec[op_idx..op_idx + 2.min(dep_spec.len() - op_idx)];
            tracing::info!("Context: After incomplete version operator={}", operator);
            return CompletionContext::VersionNumber(
                package_name.to_string(),
                operator.to_string(),
            );
        }
    }

    // If we have some text but no special characters, we're still typing the package name
    if cursor_pos <= dep_spec.len()
        && !dep_spec.contains(|c: char| matches!(c, '[' | ']' | '=' | '>' | '<' | '!' | '~'))
    {
        return CompletionContext::PackageName;
    }

    // Default to after package name
    CompletionContext::AfterPackageName(dep_spec.trim().to_string())
}

async fn complete_package_spec(
    dep_spec: &str,
    dep_string: &tombi_document_tree::String,
    position: tombi_text::Position,
    completion_hint: Option<CompletionHint>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    let string_start = dep_string.range().start;
    let cursor_offset = position - string_start;
    let cursor_pos = cursor_offset.column as usize;

    let context = analyze_completion_context(dep_spec, cursor_pos);

    tracing::info!(
        "complete_package_spec: dep_spec={:?}, cursor_pos={}, context={:?}",
        dep_spec,
        cursor_pos,
        context
    );

    match context {
        CompletionContext::PackageName => {
            // TODO: Could provide package name suggestions from PyPI search
            Ok(None)
        }
        CompletionContext::AfterPackageName(package_name) => {
            // Suggest opening bracket for features or version operators
            let mut items = vec![];

            // Suggest features syntax
            items.push(CompletionContent {
                label: "[".to_string(),
                kind: CompletionKind::String,
                emoji_icon: Some('🐍'),
                priority: tombi_extension::CompletionContentPriority::Custom(
                    "10__features__".to_string(),
                ),
                detail: Some("Specify package features".to_string()),
                documentation: Some("Add package features/extras".to_string()),
                filter_text: None,
                schema_url: None,
                deprecated: None,
                edit: Some(tombi_extension::CompletionEdit {
                    text_edit: tower_lsp::lsp_types::CompletionTextEdit::Edit(
                        tower_lsp::lsp_types::TextEdit {
                            range: tombi_text::Range::at(position).into(),
                            new_text: "[".to_string(),
                        },
                    ),
                    insert_text_format: Some(tower_lsp::lsp_types::InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: None,
                }),
                preselect: None,
            });

            // Suggest version operators
            let operators = vec![
                ("==", "Exact version match"),
                (">=", "Greater than or equal to"),
                ("<=", "Less than or equal to"),
                (">", "Greater than"),
                ("<", "Less than"),
                ("~=", "Compatible release"),
                ("!=", "Not equal to"),
            ];

            for (i, (op, desc)) in operators.iter().enumerate() {
                items.push(CompletionContent {
                    label: op.to_string(),
                    kind: CompletionKind::String,
                    emoji_icon: Some('📌'),
                    priority: tombi_extension::CompletionContentPriority::Custom(format!(
                        "10__version_{:02}__",
                        i
                    )),
                    detail: Some(desc.to_string()),
                    documentation: None,
                    filter_text: None,
                    schema_url: None,
                    deprecated: None,
                    edit: Some(tombi_extension::CompletionEdit {
                        text_edit: tower_lsp::lsp_types::CompletionTextEdit::Edit(
                            tower_lsp::lsp_types::TextEdit {
                                range: tombi_text::Range::at(position).into(),
                                new_text: op.to_string(),
                            },
                        ),
                        insert_text_format: Some(
                            tower_lsp::lsp_types::InsertTextFormat::PLAIN_TEXT,
                        ),
                        additional_text_edits: None,
                    }),
                    preselect: None,
                });
            }

            Ok(Some(items))
        }
        CompletionContext::FeatureName(package_name) => {
            // Fetch features from PyPI
            let mut items = vec![];

            if let Some(features) = fetch_package_features(&package_name, None).await {
                for (i, feature) in features.iter().enumerate() {
                    items.push(CompletionContent {
                        label: feature.clone(),
                        kind: CompletionKind::String,
                        emoji_icon: Some('🔧'),
                        priority: tombi_extension::CompletionContentPriority::Custom(format!(
                            "10__feature_{:03}__",
                            i
                        )),
                        detail: Some("Package extra".to_string()),
                        documentation: None,
                        filter_text: None,
                        schema_url: None,
                        deprecated: None,
                        edit: Some(tombi_extension::CompletionEdit {
                            text_edit: tower_lsp::lsp_types::CompletionTextEdit::Edit(
                                tower_lsp::lsp_types::TextEdit {
                                    range: tombi_text::Range::at(position).into(),
                                    new_text: feature.clone(),
                                },
                            ),
                            insert_text_format: Some(
                                tower_lsp::lsp_types::InsertTextFormat::PLAIN_TEXT,
                            ),
                            additional_text_edits: None,
                        }),
                        preselect: None,
                    });
                }
            }

            // Always suggest closing bracket
            items.push(CompletionContent {
                label: "]".to_string(),
                kind: CompletionKind::String,
                emoji_icon: Some('🐍'),
                priority: tombi_extension::CompletionContentPriority::Custom(
                    "99__close_bracket__".to_string(),
                ),
                detail: Some("Close features list".to_string()),
                documentation: None,
                filter_text: None,
                schema_url: None,
                deprecated: None,
                edit: Some(tombi_extension::CompletionEdit {
                    text_edit: tower_lsp::lsp_types::CompletionTextEdit::Edit(
                        tower_lsp::lsp_types::TextEdit {
                            range: tombi_text::Range::at(position).into(),
                            new_text: "]".to_string(),
                        },
                    ),
                    insert_text_format: Some(tower_lsp::lsp_types::InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: None,
                }),
                preselect: None,
            });

            Ok(Some(items))
        }
        CompletionContext::VersionOperator(package_name) => {
            // Already showing an operator, don't suggest more
            Ok(None)
        }
        CompletionContext::VersionNumber(package_name, operator) => {
            // Fetch and suggest version numbers
            if let Some(versions) = fetch_package_versions(&package_name).await {
                let items = versions
                    .into_iter()
                    .sorted_by(|a, b| tombi_version_sort::version_sort(a, b))
                    .rev()
                    .take(50)
                    .enumerate()
                    .map(|(i, ver)| {
                        let new_dep_spec = format!("{package_name}{operator}{ver}");
                        CompletionContent {
                            label: ver.clone(),
                            kind: CompletionKind::String,
                            emoji_icon: Some('🐍'),
                            priority: tombi_extension::CompletionContentPriority::Custom(format!(
                                "10__pypi_{i:>03}__",
                            )),
                            detail: Some("Package version".to_string()),
                            documentation: None,
                            filter_text: None,
                            schema_url: None,
                            deprecated: None,
                            edit: Some(tombi_extension::CompletionEdit {
                                text_edit: tower_lsp::lsp_types::CompletionTextEdit::Edit(
                                    tower_lsp::lsp_types::TextEdit {
                                        range: dep_string.unquoted_range().into(),
                                        new_text: new_dep_spec,
                                    },
                                ),
                                insert_text_format: Some(
                                    tower_lsp::lsp_types::InsertTextFormat::PLAIN_TEXT,
                                ),
                                additional_text_edits: None,
                            }),
                            preselect: None,
                        }
                    })
                    .collect();
                Ok(Some(items))
            } else {
                Ok(None)
            }
        }
    }
}

async fn fetch_package_versions(package_name: &str) -> Option<Vec<String>> {
    tracing::info!("Fetching versions for package: {}", package_name);
    let url = format!("https://pypi.org/pypi/{}/json", package_name);
    let client = HttpClient::new();
    let bytes = match client
        .get_bytes(&url)
        .await
        .map_err(|e| format!("http error: {e:?}"))
    {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("Failed to fetch package versions from {url}: {e}");
            return None;
        }
    };

    let resp: PyPiProjectResponse = match serde_json::from_slice(&bytes) {
        Ok(resp) => resp,
        Err(e) => {
            tracing::error!("Failed to parse package versions response: {e}");
            return None;
        }
    };

    // Filter out yanked versions
    let versions: Vec<String> = resp
        .releases
        .into_iter()
        .filter(|(_, releases)| !releases.iter().all(|r| r.yanked))
        .map(|(version, _)| version)
        .collect();

    tracing::info!(
        "Found {} versions for package {}",
        versions.len(),
        package_name
    );
    Some(versions)
}

fn fetch_package_features<'a: 'b, 'b>(
    package_name: &'a str,
    version: Option<&'a str>,
) -> BoxFuture<'b, Option<Vec<String>>> {
    Box::pin(async move {
        tracing::info!(
            "Fetching features for package: {} version: {:?}",
            package_name,
            version
        );

        let url = if let Some(ver) = version {
            format!("https://pypi.org/pypi/{}/{}/json", package_name, ver)
        } else {
            format!("https://pypi.org/pypi/{}/json", package_name)
        };

        let client = HttpClient::new();
        let bytes = match client
            .get_bytes(&url)
            .await
            .map_err(|e| format!("http error: {e:?}"))
        {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("Failed to fetch package info from {url}: {e}");
                return None;
            }
        };

        if let Some(ver) = version {
            // Fetch specific version info
            let resp: PyPiVersionResponse = match serde_json::from_slice(&bytes) {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::error!("Failed to parse version response: {e}");
                    return None;
                }
            };

            let features = resp.info.provides_extra.unwrap_or_default();
            tracing::info!(
                "Found {} features for package {} version {}",
                features.len(),
                package_name,
                ver
            );
            Some(features)
        } else {
            // Fetch latest version info
            let resp: PyPiProjectResponse = match serde_json::from_slice(&bytes) {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::error!("Failed to parse project response: {e}");
                    return None;
                }
            };

            // Try to get provides_extra from the latest version
            let latest_version = resp.info.version;
            tracing::info!(
                "Latest version for {} is {}, fetching its features",
                package_name,
                latest_version
            );

            // Fetch the specific version info to get provides_extra
            fetch_package_features(package_name, Some(&latest_version)).await
        }
    })
    .boxed()
}
