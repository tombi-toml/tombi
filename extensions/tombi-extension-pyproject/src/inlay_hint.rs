use std::{collections::BTreeSet, path::Path, str::FromStr};

use pep508_rs::{
    VerbatimUrl, VersionOrUrl,
    pep440_rs::{Operator, Version},
};
use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{TryIntoDocumentTree, Value, dig_keys};
use tombi_extension::InlayHint;

const RESOLVED_VERSION_TOOLTIP: &str = "Resolved version in uv.lock";

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
    let features = features.cloned();

    let text_document_uri = text_document_uri.clone();
    let document_tree = document_tree.clone();

    tokio::task::spawn_blocking(move || {
        inlay_hint_impl(
            &text_document_uri,
            &document_tree,
            visible_range,
            toml_version,
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

    let Some(current_package) = current_package(document_tree) else {
        return Ok(None);
    };

    let visible_dependency_hints = collect_dependency_hints(document_tree)
        .into_iter()
        .filter(|hint| {
            tombi_text::Range::at(hint.dependency.range().end).intersects(visible_range)
        })
        .collect::<Vec<_>>();

    if visible_dependency_hints.is_empty() {
        return Ok(None);
    }

    let Some(uv_lock) = load_uv_lock(&pyproject_toml_path, toml_version) else {
        return Ok(None);
    };

    let hints = visible_dependency_hints
        .into_iter()
        .filter_map(|hint| {
            let resolved_version = uv_lock.dependency_version_for_package(
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

fn load_uv_lock(pyproject_toml_path: &Path, toml_version: TomlVersion) -> Option<UvLock> {
    let uv_lock_path = find_uv_lock_path(pyproject_toml_path)?;
    let uv_lock_text = std::fs::read_to_string(uv_lock_path).ok()?;
    let root = tombi_ast::Root::cast(tombi_parser::parse(&uv_lock_text).into_syntax_node())?;
    let document_tree = root.try_into_document_tree(toml_version).ok()?;

    UvLock::from_document_tree(&document_tree)
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

    fn package(&self, package_name: &str, package_version: &str) -> Option<&UvLockPackage> {
        self.packages.iter().find(|package| {
            package.name == package_name && normalize_version(&package.version) == package_version
        })
    }

    fn unique_package_version(&self, dependency_name: &str) -> Option<String> {
        let versions = self
            .packages
            .iter()
            .filter(|package| package.name == dependency_name)
            .map(|package| package.version.clone())
            .collect::<BTreeSet<_>>();

        (versions.len() == 1)
            .then(|| versions.into_iter().next())
            .flatten()
    }

    fn resolved_dependency_version(&self, dependency: &UvLockDependency) -> Option<String> {
        dependency
            .version
            .clone()
            .or_else(|| self.unique_package_version(&dependency.name))
    }

    fn dependency_version_for_package(
        &self,
        package_name: &str,
        package_version: &str,
        dependency_name: &str,
    ) -> Option<String> {
        let package = self.package(package_name, package_version)?;
        let versions = package
            .direct_dependencies
            .iter()
            .filter(|dependency| dependency.name == dependency_name)
            .filter_map(|dependency| self.resolved_dependency_version(dependency))
            .collect::<BTreeSet<_>>();

        (versions.len() == 1)
            .then(|| versions.into_iter().next())
            .flatten()
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
