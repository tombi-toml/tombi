use std::{
    borrow::Borrow,
    path::{Path, PathBuf},
    str::FromStr,
};

use pep508_rs::{
    VerbatimUrl, VersionOrUrl,
    pep440_rs::{Operator, Version},
};
use serde::{Deserialize, Serialize};
use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{TryIntoDocumentTree, Value, dig_keys};
use tombi_extension::{InlayHint, file_cache_version, get_or_load_json};
use tombi_hashmap::{HashMap, HashSet};

const RESOLVED_VERSION_TOOLTIP: &str = "Resolved version in uv.lock";
const PYPROJECT_EXTENSION_ID: &str = "tombi-toml/pyproject";
const INLAY_HINT_LOCKFILE_KEY: &str = "inlay_hint.lockfile";

#[derive(Debug)]
struct UvLock {
    packages: Vec<UvLockPackage>,
}

#[derive(Debug)]
struct UvLockPackage {
    name: String,
    version: String,
    direct_dependencies: Vec<UvLockDependency>,
}

#[derive(Debug)]
struct UvLockDependency {
    name: String,
    version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
struct ProjectName(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
struct ProjectVersion(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
struct DependencyProjectName(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResolvedProjectDependencyVersion {
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectResolvedDependencies {
    by_dependency: tombi_hashmap::HashMap<DependencyProjectName, ResolvedProjectDependencyVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UvLockInlayCacheData {
    projects: tombi_hashmap::HashMap<
        ProjectName,
        tombi_hashmap::HashMap<ProjectVersion, ProjectResolvedDependencies>,
    >,
}

struct PyprojectDependencyHint<'a> {
    dependency: &'a tombi_document_tree::String,
    requirement: pep508_rs::Requirement<VerbatimUrl>,
}

pub async fn inlay_hint(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
    toml_version: TomlVersion,
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> Result<Option<Vec<InlayHint>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    if !pyproject_inlay_hint_root_enabled(features)
        || !pyproject_inlay_hint_dependency_version_enabled(features)
    {
        return Ok(None);
    }

    let Ok(pyproject_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    let features = features.cloned();

    let text_document_uri = text_document_uri.clone();
    let document_tree = document_tree.clone();
    let uv_lock_cache = load_uv_lock_cache(&pyproject_toml_path, toml_version).await;

    tokio::task::spawn_blocking(move || {
        inlay_hint_impl(
            &text_document_uri,
            &document_tree,
            visible_range,
            uv_lock_cache,
            features.as_ref(),
        )
    })
    .await
    .map_err(|_| tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::InternalError))?
}

fn inlay_hint_impl(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
    uv_lock_cache: Option<UvLockInlayCacheData>,
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> Result<Option<Vec<InlayHint>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    if !pyproject_inlay_hint_root_enabled(features)
        || !pyproject_inlay_hint_dependency_version_enabled(features)
    {
        return Ok(None);
    }

    let Some(current_package) = current_package(document_tree) else {
        return Ok(None);
    };

    let visible_dependency_hints = collect_dependency_hints(document_tree)
        .into_iter()
        .filter(|hint| tombi_text::Range::at(hint.dependency.range().end).intersects(visible_range))
        .collect::<Vec<_>>();

    if visible_dependency_hints.is_empty() {
        return Ok(None);
    }

    let Some(uv_lock_cache) = uv_lock_cache else {
        return Ok(None);
    };

    let hints = visible_dependency_hints
        .into_iter()
        .filter_map(|hint| {
            let resolved_version = uv_lock_cache.resolved_dependency_version(
                &current_package.name,
                &current_package.version,
                hint.requirement.name.as_ref(),
            )?;

            let current_version = exact_pinned_version(&hint.requirement);
            let label = version_hint_label(current_version.as_deref(), &resolved_version)?;

            Some(InlayHint {
                position: hint.dependency.range().end,
                label,
                kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
                tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
                padding_left: Some(true),
                padding_right: Some(false),
            })
        })
        .collect::<Vec<_>>();

    if hints.is_empty() {
        Ok(None)
    } else {
        Ok(Some(hints))
    }
}

fn pyproject_inlay_hint_root_enabled(
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> bool {
    features.map_or(
        true,
        tombi_config::PyprojectExtensionFeatures::inlay_hint_enabled,
    )
}

fn pyproject_inlay_hint_dependency_version_enabled(
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> bool {
    features.map_or(
        true,
        tombi_config::PyprojectExtensionFeatures::dependency_version_inlay_hint_enabled,
    )
}

struct CurrentPackage {
    name: String,
    version: String,
}

fn current_package(document_tree: &tombi_document_tree::DocumentTree) -> Option<CurrentPackage> {
    let (_, Value::String(name)) = dig_keys(document_tree, &["project", "name"])? else {
        return None;
    };
    let (_, Value::String(version)) = dig_keys(document_tree, &["project", "version"])? else {
        return None;
    };

    Some(CurrentPackage {
        name: name.value().to_string(),
        version: normalize_version(version.value()),
    })
}

fn collect_dependency_hints<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
) -> Vec<PyprojectDependencyHint<'a>> {
    let mut hints = Vec::new();

    if let Some((_, Value::Array(dependencies))) =
        dig_keys(document_tree, &["project", "dependencies"])
    {
        hints.extend(dependencies.iter().filter_map(pyproject_dependency_hint));
    }

    if let Some((_, Value::Table(optional_dependencies))) =
        dig_keys(document_tree, &["project", "optional-dependencies"])
    {
        for value in optional_dependencies.values() {
            if let Value::Array(dependencies) = value {
                hints.extend(dependencies.iter().filter_map(pyproject_dependency_hint));
            }
        }
    }

    if let Some((_, Value::Table(dependency_groups))) =
        dig_keys(document_tree, &["dependency-groups"])
    {
        for value in dependency_groups.values() {
            if let Value::Array(dependencies) = value {
                hints.extend(dependencies.iter().filter_map(pyproject_dependency_hint));
            }
        }
    }

    hints
}

fn pyproject_dependency_hint(
    value: &tombi_document_tree::Value,
) -> Option<PyprojectDependencyHint<'_>> {
    let Value::String(dependency) = value else {
        return None;
    };

    let requirement = pep508_rs::Requirement::<VerbatimUrl>::from_str(dependency.value()).ok()?;
    Some(PyprojectDependencyHint {
        dependency,
        requirement,
    })
}

fn version_hint_label(current_version: Option<&str>, resolved_version: &str) -> Option<String> {
    if current_version == Some(resolved_version) {
        return None;
    }

    Some(format!(r#" → "{resolved_version}""#))
}

fn exact_pinned_version(requirement: &pep508_rs::Requirement<VerbatimUrl>) -> Option<String> {
    let VersionOrUrl::VersionSpecifier(specifiers) = requirement.version_or_url.as_ref()? else {
        return None;
    };
    if specifiers.len() != 1 {
        return None;
    }

    let specifier = &specifiers[0];
    match specifier.operator() {
        Operator::Equal | Operator::ExactEqual => Some(specifier.version().to_string()),
        _ => None,
    }
}

async fn load_uv_lock_cache(
    pyproject_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<UvLockInlayCacheData> {
    let uv_lock_path = find_uv_lock_path(pyproject_toml_path)?;
    let cache_key = uv_lock_cache_key(&uv_lock_path);
    let cache_version = file_cache_version(&uv_lock_path);

    let cache_value = get_or_load_json(&cache_key, cache_version, {
        let uv_lock_path = uv_lock_path.clone();
        move || async move { load_uv_lock_cache_json(uv_lock_path, toml_version).await }
    })
    .await?;

    UvLockInlayCacheData::deserialize(cache_value.as_ref()).ok()
}

fn find_uv_lock_path(pyproject_toml_path: &Path) -> Option<std::path::PathBuf> {
    let mut current_dir = pyproject_toml_path.parent()?;

    loop {
        let candidate = current_dir.join("uv.lock");
        if candidate.is_file() {
            return candidate.canonicalize().ok().or(Some(candidate));
        }

        current_dir = current_dir.parent()?;
    }
}

fn normalize_version(version: &str) -> String {
    Version::from_str(version)
        .map(|version| version.to_string())
        .unwrap_or_else(|_| version.to_string())
}

async fn load_uv_lock_cache_json(
    uv_lock_path: PathBuf,
    toml_version: TomlVersion,
) -> Option<serde_json::Value> {
    let uv_lock_text = tokio::fs::read_to_string(&uv_lock_path).await.ok()?;
    tokio::task::spawn_blocking(move || parse_uv_lock_cache_json(uv_lock_text, toml_version))
        .await
        .ok()
        .flatten()
}

fn parse_uv_lock_cache_json(
    uv_lock_text: String,
    toml_version: TomlVersion,
) -> Option<serde_json::Value> {
    let root = tombi_ast::Root::cast(tombi_parser::parse(&uv_lock_text).into_syntax_node())?;
    let document_tree = root.try_into_document_tree(toml_version).ok()?;
    let uv_lock = UvLock::from_document_tree(&document_tree)?;

    serde_json::to_value(uv_lock.into_inlay_cache_data()).ok()
}

fn uv_lock_cache_key(uv_lock_path: &Path) -> String {
    format!(
        "{PYPROJECT_EXTENSION_ID}:{INLAY_HINT_LOCKFILE_KEY}:{}",
        uv_lock_path.display()
    )
}

impl UvLock {
    fn from_document_tree(document_tree: &tombi_document_tree::DocumentTree) -> Option<Self> {
        let (_, Value::Array(packages)) = dig_keys(document_tree, &["package"])? else {
            return None;
        };

        Some(Self {
            packages: packages
                .iter()
                .filter_map(UvLockPackage::from_value)
                .collect(),
        })
    }

    fn into_inlay_cache_data(self) -> UvLockInlayCacheData {
        let unique_package_versions = self.unique_package_versions();
        let mut projects = HashMap::new();

        for package in &self.packages {
            projects
                .entry(ProjectName::new(&package.name))
                .or_insert_with(HashMap::new)
                .insert(
                    ProjectVersion::new(&normalize_version(&package.version)),
                    package.resolved_dependencies(&unique_package_versions),
                );
        }

        UvLockInlayCacheData { projects }
    }

    fn unique_package_versions(&self) -> HashMap<String, Option<String>> {
        let mut package_versions = HashMap::<String, HashSet<String>>::new();

        for package in &self.packages {
            package_versions
                .entry(package.name.clone())
                .or_default()
                .insert(package.version.clone());
        }

        package_versions
            .into_iter()
            .map(|(package_name, versions)| {
                let version = (versions.len() == 1)
                    .then(|| versions.into_iter().next())
                    .flatten();
                (package_name, version)
            })
            .collect()
    }
}

impl UvLockPackage {
    fn from_value(value: &Value) -> Option<Self> {
        let Value::Table(table) = value else {
            return None;
        };

        let name = match table.get("name") {
            Some(Value::String(name)) => name.value().to_string(),
            _ => return None,
        };
        let version = match table.get("version") {
            Some(Value::String(version)) => version.value().to_string(),
            _ => return None,
        };

        let mut direct_dependencies = Vec::new();
        direct_dependencies.extend(uv_dependencies(table.get("dependencies")));
        direct_dependencies.extend(uv_dependency_groups(table.get("optional-dependencies")));
        direct_dependencies.extend(uv_dependency_groups(table.get("dev-dependencies")));

        Some(Self {
            name,
            version,
            direct_dependencies,
        })
    }

    fn resolved_dependencies(
        &self,
        unique_package_versions: &HashMap<String, Option<String>>,
    ) -> ProjectResolvedDependencies {
        let dependency_names = self
            .direct_dependencies
            .iter()
            .map(|dependency| dependency.name.clone())
            .collect::<HashSet<_>>();

        let by_dependency = dependency_names
            .into_iter()
            .filter_map(|dependency_name| {
                let resolved_version =
                    self.resolved_dependency_version(&dependency_name, unique_package_versions)?;
                Some((
                    DependencyProjectName::new(&dependency_name),
                    ResolvedProjectDependencyVersion::new(resolved_version),
                ))
            })
            .collect();

        ProjectResolvedDependencies { by_dependency }
    }

    fn resolved_dependency_version(
        &self,
        dependency_name: &str,
        unique_package_versions: &HashMap<String, Option<String>>,
    ) -> Option<String> {
        let versions = self
            .direct_dependencies
            .iter()
            .filter(|dependency| dependency.name == dependency_name)
            .filter_map(|dependency| {
                dependency.version.clone().or_else(|| {
                    unique_package_versions
                        .get(&dependency.name)
                        .cloned()
                        .flatten()
                })
            })
            .collect::<HashSet<_>>();

        (versions.len() == 1)
            .then(|| versions.into_iter().next())
            .flatten()
    }
}

fn uv_dependencies(value: Option<&Value>) -> Vec<UvLockDependency> {
    let Some(Value::Array(dependencies)) = value else {
        return Vec::new();
    };

    dependencies
        .iter()
        .filter_map(UvLockDependency::from_value)
        .collect()
}

fn uv_dependency_groups(value: Option<&Value>) -> Vec<UvLockDependency> {
    let Some(Value::Table(groups)) = value else {
        return Vec::new();
    };

    groups
        .values()
        .filter_map(|value| match value {
            Value::Array(dependencies) => Some(
                dependencies
                    .iter()
                    .filter_map(UvLockDependency::from_value)
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten()
        .collect()
}

impl UvLockDependency {
    fn from_value(value: &Value) -> Option<Self> {
        let Value::Table(table) = value else {
            return None;
        };

        let name = match table.get("name") {
            Some(Value::String(name)) => name.value().to_string(),
            _ => return None,
        };
        let version = match table.get("version") {
            Some(Value::String(version)) => Some(version.value().to_string()),
            _ => None,
        };

        Some(Self { name, version })
    }
}

impl ProjectName {
    fn new(project_name: &str) -> Self {
        Self(project_name.to_string())
    }
}

impl Borrow<str> for ProjectName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl ProjectVersion {
    fn new(project_version: &str) -> Self {
        Self(project_version.to_string())
    }
}

impl Borrow<str> for ProjectVersion {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl DependencyProjectName {
    fn new(dependency_name: &str) -> Self {
        Self(dependency_name.to_string())
    }
}

impl Borrow<str> for DependencyProjectName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl ResolvedProjectDependencyVersion {
    fn new(version: String) -> Self {
        Self { version }
    }
}

impl UvLockInlayCacheData {
    fn resolved_dependency_version(
        &self,
        project_name: &str,
        project_version: &str,
        dependency_name: &str,
    ) -> Option<String> {
        self.projects
            .get(project_name)
            .and_then(|versions| versions.get(project_version))
            .and_then(|dependencies| dependencies.by_dependency.get(dependency_name))
            .map(|resolved| resolved.version.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_lock_cache_data_resolves_dependency_version() {
        let uv_lock = UvLock {
            packages: vec![
                UvLockPackage {
                    name: "demo".to_string(),
                    version: "0.1.0".to_string(),
                    direct_dependencies: vec![UvLockDependency {
                        name: "pytest".to_string(),
                        version: None,
                    }],
                },
                UvLockPackage {
                    name: "pytest".to_string(),
                    version: "8.3.3".to_string(),
                    direct_dependencies: Vec::new(),
                },
            ],
        };

        let cache_data = uv_lock.into_inlay_cache_data();

        assert_eq!(
            cache_data.resolved_dependency_version("demo", "0.1.0", "pytest"),
            Some("8.3.3".to_string())
        );
    }

    #[test]
    fn uv_lock_cache_data_uses_normalized_project_version() {
        let uv_lock = UvLock {
            packages: vec![UvLockPackage {
                name: "demo".to_string(),
                version: "1.0".to_string(),
                direct_dependencies: Vec::new(),
            }],
        };

        let cache_data = uv_lock.into_inlay_cache_data();

        assert!(
            cache_data
                .projects
                .get("demo")
                .is_some_and(|versions| versions.contains_key("1.0"))
        );
    }

    #[test]
    fn uv_lock_cache_data_survives_json_roundtrip() {
        let uv_lock = UvLock {
            packages: vec![
                UvLockPackage {
                    name: "demo".to_string(),
                    version: "0.1.0".to_string(),
                    direct_dependencies: vec![UvLockDependency {
                        name: "ruff".to_string(),
                        version: Some("0.7.4".to_string()),
                    }],
                },
                UvLockPackage {
                    name: "ruff".to_string(),
                    version: "0.7.4".to_string(),
                    direct_dependencies: Vec::new(),
                },
            ],
        };

        let cache_data = uv_lock.into_inlay_cache_data();
        let roundtrip = serde_json::from_value::<UvLockInlayCacheData>(
            serde_json::to_value(&cache_data).expect("expected json value"),
        )
        .expect("expected cache data");

        assert_eq!(
            roundtrip.resolved_dependency_version("demo", "0.1.0", "ruff"),
            Some("0.7.4".to_string())
        );
    }
}
