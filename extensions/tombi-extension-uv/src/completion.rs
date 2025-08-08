use ahash::AHashMap;
use serde::Deserialize;
use tombi_config::TomlVersion;
use tombi_extension::{
    CompletionContent, CompletionContentPriority, CompletionHint, CompletionKind,
};
use tombi_future::BoxFuture;
use tombi_future::Boxable;
use tombi_schema_store::dig_accessors;
use tombi_schema_store::matches_accessors;
use tombi_schema_store::Accessor;
use tombi_schema_store::HttpClient;
use tombi_version_sort::version_sort;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use tombi_pep508::ast::AstNode;
use tombi_pep508::{Parser, SyntaxKind};
use tombi_rg_tree::TokenAtOffset;

#[derive(Debug, Deserialize)]
struct PyPiProjectResponse {
    #[allow(dead_code)]
    info: PyPiProjectInfo,
    releases: AHashMap<String, Vec<PyPiReleaseInfo>>,
}

#[derive(Debug, Deserialize)]
struct PyPiProjectInfo {
    #[allow(dead_code)]
    name: String,
    version: String,
    #[serde(default)]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    version: String,
    #[serde(default)]
    #[allow(dead_code)]
    requires_dist: Option<Vec<String>>,
    #[serde(default)]
    provides_extra: Option<Vec<String>>,
}

pub async fn completion(
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
    _toml_version: TomlVersion,
    completion_hint: Option<CompletionHint>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if !text_document.uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    // Handle various dependency sections
    if matches_accessors!(accessors, ["project", "dependencies", _])
        || matches_accessors!(accessors, ["project", "optional-dependencies", _, _])
        || matches_accessors!(accessors, ["dependency-groups", _, _])
        || matches_accessors!(accessors, ["build-system", "requires", _])
    {
        if let Some((_, tombi_document_tree::Value::String(dep_spec))) =
            dig_accessors(document_tree, accessors)
        {
            return complete_package_spec(dep_spec, position, completion_hint).await;
        }
    }

    Ok(None)
}

async fn complete_package_spec(
    dep_spec: &tombi_document_tree::String,
    position: tombi_text::Position,
    _completion_hint: Option<CompletionHint>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    let mut completions = Vec::new();

    // Check if position is within the string's range
    let dep_spec_range = dep_spec.range();
    if !dep_spec_range.contains(position) {
        return Ok(None);
    }

    // Convert absolute position to relative position within the string content
    // We need to account for the fact that the range includes quotes but the value doesn't
    let relative_column = if position.line == dep_spec_range.start.line {
        // Calculate the offset within the string value (excluding quotes)
        // The range starts at the opening quote, so we subtract 1 for the quote
        position
            .column
            .saturating_sub(dep_spec_range.start.column + 1)
    } else {
        // If on a different line (shouldn't happen for dependency specs), use column as-is
        position.column
    };

    let relative_position = tombi_text::Position::new(0, relative_column);

    // Parse the dependency specification to AST
    let parser = Parser::new(dep_spec.value());
    let (ast_root, _errors) = parser.parse_ast();

    // Get root node from AST
    let root = tombi_pep508::ast::Root::cast(ast_root).unwrap();

    // Find the token at the cursor position
    let token_at_cursor = root.syntax().token_at_position(relative_position);

    // Check if we have a requirement node
    let Some(requirement) = root.requirement() else {
        return Ok(None);
    };

    // Get package name if it exists
    let package_name = requirement.package_name();

    tracing::info!(
        "Completion for '{}' at rel pos {:?}, token: {:?}, has_package: {}",
        dep_spec.value(),
        relative_position,
        token_at_cursor,
        package_name.is_some()
    );

    // Determine what kind of completion we need based on AST structure
    if let Some(name_node) = package_name {
        let name_range = name_node.syntax().range();

        // Check if cursor is after the package name
        if relative_position > name_range.end {
            // Check what comes after the package name
            let extras_list = requirement.extras_list();
            let version_spec = requirement.version_spec();
            let url_spec = requirement.url_spec();
            let marker_expr = requirement.marker();

            // If we're in extras brackets
            if let Some(ref extras) = extras_list {
                let extras_range = extras.syntax().range();
                if extras_range.contains(relative_position) {
                    // We're inside extras - provide extra completions
                    let package_name_str = name_node.syntax().text().to_string();

                    // Check if we're after a comma
                    let after_comma = matches!(
                        &token_at_cursor,
                        TokenAtOffset::Single(token) if token.kind() == SyntaxKind::COMMA
                    ) || matches!(
                        &token_at_cursor,
                        TokenAtOffset::Between(left, _) if left.kind() == SyntaxKind::COMMA
                    );

                    // Get existing extras
                    let existing_extras: Vec<String> = extras.extras().collect();

                    // Fetch and suggest available extras
                    if let Some(features) = fetch_package_features(&package_name_str, None).await {
                        for feature in features {
                            if !existing_extras.contains(&feature) {
                                completions.push(CompletionContent {
                                    label: feature.clone(),
                                    kind: CompletionKind::String,
                                    emoji_icon: None,
                                    priority: CompletionContentPriority::Default,
                                    detail: Some(format!("Extra: {}", feature)),
                                    documentation: None,
                                    filter_text: None,
                                    schema_url: None,
                                    deprecated: None,
                                    edit: None,
                                    preselect: None,
                                });
                            }
                        }
                    }

                    // Suggest closing bracket if not after comma
                    if !after_comma {
                        completions.push(CompletionContent {
                            label: "]".to_string(),
                            kind: CompletionKind::String,
                            emoji_icon: None,
                            priority: CompletionContentPriority::Default,
                            detail: Some("Close extras list".to_string()),
                            documentation: None,
                            filter_text: None,
                            schema_url: None,
                            deprecated: None,
                            edit: None,
                            preselect: None,
                        });
                    }

                    if !completions.is_empty() {
                        return Ok(Some(completions));
                    }
                }
            }

            // If we're in a version spec
            if let Some(ref version) = version_spec {
                let version_range = version.syntax().range();
                if version_range.contains(relative_position) {
                    // Check if we're after the operator
                    // Check if we have a version operator token
                    let has_operator = version.syntax().children_with_tokens().any(|child| {
                        matches!(
                            child.kind(),
                            SyntaxKind::EQ_EQ
                                | SyntaxKind::GTE
                                | SyntaxKind::TILDE_EQ
                                | SyntaxKind::GT
                                | SyntaxKind::LT
                                | SyntaxKind::LTE
                                | SyntaxKind::NOT_EQ
                        )
                    });

                    if has_operator {
                        // Find the position of the operator
                        let op_end = version
                            .syntax()
                            .children_with_tokens()
                            .find(|child| {
                                matches!(
                                    child.kind(),
                                    SyntaxKind::EQ_EQ
                                        | SyntaxKind::GTE
                                        | SyntaxKind::TILDE_EQ
                                        | SyntaxKind::GT
                                        | SyntaxKind::LT
                                        | SyntaxKind::LTE
                                        | SyntaxKind::NOT_EQ
                                )
                            })
                            .map(|op| op.range().end)
                            .unwrap_or(version_range.start);

                        if relative_position > op_end {
                            // We're after the operator, suggest versions
                            let package_name_str = name_node.syntax().text().to_string();
                            if let Some(mut versions) =
                                fetch_package_versions(&package_name_str).await
                            {
                                versions.sort_by(|a, b| version_sort(a, b));
                                for (i, version) in versions.iter().rev().take(10).enumerate() {
                                    completions.push(CompletionContent {
                                        label: version.clone(),
                                        kind: CompletionKind::String,
                                        emoji_icon: None,
                                        priority: CompletionContentPriority::Default,
                                        detail: Some(if i == 0 {
                                            "Latest version".to_string()
                                        } else {
                                            format!("Version {}", version)
                                        }),
                                        documentation: None,
                                        filter_text: None,
                                        schema_url: None,
                                        deprecated: None,
                                        edit: None,
                                        preselect: None,
                                    });
                                }
                            }

                            if !completions.is_empty() {
                                return Ok(Some(completions));
                            }
                        }
                    }
                }
            }

            // If we have a marker expression
            if let Some(ref marker) = marker_expr {
                let marker_range = marker.syntax().range();
                if marker_range.contains(relative_position) {
                    // In marker expression - suggest marker operators
                    completions.push(CompletionContent {
                        label: "and".to_string(),
                        kind: CompletionKind::String,
                        emoji_icon: None,
                        priority: CompletionContentPriority::Default,
                        detail: Some("Logical AND".to_string()),
                        documentation: None,
                        filter_text: None,
                        schema_url: None,
                        deprecated: None,
                        edit: None,
                        preselect: None,
                    });
                    completions.push(CompletionContent {
                        label: "or".to_string(),
                        kind: CompletionKind::String,
                        emoji_icon: None,
                        priority: CompletionContentPriority::Default,
                        detail: Some("Logical OR".to_string()),
                        documentation: None,
                        filter_text: None,
                        schema_url: None,
                        deprecated: None,
                        edit: None,
                        preselect: None,
                    });

                    if !completions.is_empty() {
                        return Ok(Some(completions));
                    }
                }
            }

            // Check for semicolon (marker start)
            let has_semicolon = token_at_cursor
                .clone()
                .into_iter()
                .any(|t| t.kind() == SyntaxKind::SEMICOLON);

            if has_semicolon && marker_expr.is_none() {
                // After semicolon, suggest marker variables
                completions.push(CompletionContent {
                    label: "python_version".to_string(),
                    kind: CompletionKind::Key,
                    emoji_icon: None,
                    priority: CompletionContentPriority::Default,
                    detail: Some("Python version marker".to_string()),
                    documentation: None,
                    filter_text: None,
                    schema_url: None,
                    deprecated: None,
                    edit: None,
                    preselect: None,
                });
                completions.push(CompletionContent {
                    label: "sys_platform".to_string(),
                    kind: CompletionKind::Key,
                    emoji_icon: None,
                    priority: CompletionContentPriority::Default,
                    detail: Some("System platform marker".to_string()),
                    documentation: None,
                    filter_text: None,
                    schema_url: None,
                    deprecated: None,
                    edit: None,
                    preselect: None,
                });
                completions.push(CompletionContent {
                    label: "platform_machine".to_string(),
                    kind: CompletionKind::Key,
                    emoji_icon: None,
                    priority: CompletionContentPriority::Default,
                    detail: Some("Platform machine marker".to_string()),
                    documentation: None,
                    filter_text: None,
                    schema_url: None,
                    deprecated: None,
                    edit: None,
                    preselect: None,
                });

                if !completions.is_empty() {
                    return Ok(Some(completions));
                }
            }

            // If nothing specific matched, we're after package name
            // Suggest operators and brackets
            if extras_list.is_none()
                && version_spec.is_none()
                && url_spec.is_none()
                && marker_expr.is_none()
            {
                // Suggest version operators
                completions.push(CompletionContent {
                    label: ">=".to_string(),
                    kind: CompletionKind::String,
                    emoji_icon: None,
                    priority: CompletionContentPriority::Default,
                    detail: Some("Greater than or equal to".to_string()),
                    documentation: None,
                    filter_text: None,
                    schema_url: None,
                    deprecated: None,
                    edit: None,
                    preselect: None,
                });
                completions.push(CompletionContent {
                    label: "==".to_string(),
                    kind: CompletionKind::String,
                    emoji_icon: None,
                    priority: CompletionContentPriority::Default,
                    detail: Some("Exactly equal to".to_string()),
                    documentation: None,
                    filter_text: None,
                    schema_url: None,
                    deprecated: None,
                    edit: None,
                    preselect: None,
                });
                completions.push(CompletionContent {
                    label: "~=".to_string(),
                    kind: CompletionKind::String,
                    emoji_icon: None,
                    priority: CompletionContentPriority::Default,
                    detail: Some("Compatible release".to_string()),
                    documentation: None,
                    filter_text: None,
                    schema_url: None,
                    deprecated: None,
                    edit: None,
                    preselect: None,
                });
                completions.push(CompletionContent {
                    label: "[".to_string(),
                    kind: CompletionKind::String,
                    emoji_icon: None,
                    priority: CompletionContentPriority::Default,
                    detail: Some("Add extras".to_string()),
                    documentation: None,
                    filter_text: None,
                    schema_url: None,
                    deprecated: None,
                    edit: None,
                    preselect: None,
                });
                completions.push(CompletionContent {
                    label: ";".to_string(),
                    kind: CompletionKind::String,
                    emoji_icon: None,
                    priority: CompletionContentPriority::Default,
                    detail: Some("Add environment marker".to_string()),
                    documentation: None,
                    filter_text: None,
                    schema_url: None,
                    deprecated: None,
                    edit: None,
                    preselect: None,
                });
            }
        }
    } else {
        // No package name yet - don't provide completions
        return Ok(None);
    }

    if completions.is_empty() {
        return Ok(None);
    }

    Ok(Some(completions))
}

#[allow(dead_code)]
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
