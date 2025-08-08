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
use tombi_pep508::{CompletionContext, Parser};

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

    // Handle [project] dependencies
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

    // Parse the dependency specification to AST
    let parser = Parser::new(dep_spec.value());
    let (ast_root, _errors) = parser.parse_ast();

    // Get root node from AST
    let root = tombi_pep508::ast::Root::cast(ast_root).unwrap();

    // Get completion context from AST at cursor position
    let context = CompletionContext::from_ast(&root, position);

    tracing::info!("Completion context: {:?}", context);

    match context {
        None | Some(CompletionContext::Empty) => {}
        Some(CompletionContext::AfterPackageName { .. }) => {
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
        Some(CompletionContext::InExtras { after_comma, .. }) => {
            // Get package name and existing extras
            let package_name = context.as_ref().unwrap().package_name().unwrap_or_default();
            let existing_extras = context.as_ref().unwrap().existing_extras();

            // Fetch available extras for the package
            if let Some(features) = fetch_package_features(&package_name, None).await {
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

            // Also suggest closing the extras list
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
        }
        Some(CompletionContext::InVersionSpec { .. }) => {
            // Get package name
            let package_name = context.as_ref().unwrap().package_name().unwrap_or_default();

            // Fetch and suggest versions
            if let Some(mut versions) = fetch_package_versions(&package_name).await {
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
        }
        Some(CompletionContext::AfterSemicolon { .. }) => {
            // Suggest common environment markers
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
        }
        Some(CompletionContext::InMarkerExpression { .. }) => {
            // Suggest marker operators and values
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
        }
        Some(CompletionContext::AfterAt { .. }) => {
            // URL completion - could add common URL prefixes
        }
        Some(CompletionContext::InUrl { .. }) => {
            // URL path completion
        }
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
