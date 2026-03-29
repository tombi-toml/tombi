use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{TryIntoDocumentTree, Value, dig_keys};
use tombi_extension::InlayHint;

use crate::{
    dependency_package_name, find_workspace_cargo_toml, get_workspace_path, load_cargo_toml,
    workspace::{extract_exclude_patterns, find_package_cargo_toml_paths},
};

const RESOLVED_VERSION_TOOLTIP: &str = "Resolved version in Cargo.lock";
const WORKSPACE_PACKAGE_INHERITED_VALUE_TOOLTIP: &str = "Inherited value from workspace.package";
const MAX_WORKSPACE_VALUE_HINT_CHARS: usize = 80;
const WORKSPACE_PACKAGE_ITEMS: [&str; 16] = [
    "authors",
    "categories",
    "description",
    "documentation",
    "edition",
    "exclude",
    "homepage",
    "include",
    "keywords",
    "license-file",
    "license",
    "publish",
    "readme",
    "repository",
    "rust-version",
    "version",
];

enum CargoInlayHintFeature {
    DependencyVersion,
}

#[derive(Debug)]
struct CargoLock {
    packages: Vec<CargoLockPackage>,
}

#[derive(Debug)]
struct CargoLockPackage {
    name: String,
    version: String,
    dependencies: Vec<CargoLockDependency>,
}

#[derive(Debug)]
struct CargoLockDependency {
    name: String,
    version: Option<String>,
}

struct DependencyVersionHint {
    dependency_name: String,
    position: tombi_text::Position,
    current_version: Option<String>,
    always_show: bool,
}

struct CurrentPackage<'a> {
    name: &'a str,
    version: String,
}

struct WorkspaceMemberPackage {
    name: String,
    version: String,
}

enum WorkspaceDocumentTree<'a> {
    Current(&'a tombi_document_tree::DocumentTree),
    External(tombi_document_tree::DocumentTree),
}

impl WorkspaceDocumentTree<'_> {
    fn as_tree(&self) -> &tombi_document_tree::DocumentTree {
        match self {
            Self::Current(document_tree) => document_tree,
            Self::External(document_tree) => document_tree,
        }
    }
}

pub async fn inlay_hint(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    visible_range: tombi_text::Range,
    toml_version: TomlVersion,
    _offline: bool,
    _cache_options: Option<&tombi_cache::Options>,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<InlayHint>>, tower_lsp::jsonrpc::Error> {
    let text_document_uri = text_document_uri.clone();
    let document_tree = document_tree.clone();
    let features = features.cloned();

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
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<InlayHint>>, tower_lsp::jsonrpc::Error> {
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }

    if !cargo_inlay_hint_root_enabled(features) {
        return Ok(None);
    }

    let Ok(cargo_toml_path) = text_document_uri.to_file_path() else {
        return Ok(None);
    };

    let mut hints = Vec::new();
    let mut cargo_lock = None;

    collect_workspace_package_inlay_hints(
        document_tree,
        &cargo_toml_path,
        toml_version,
        visible_range,
        &mut hints,
    );

    if cargo_inlay_hint_enabled(features, CargoInlayHintFeature::DependencyVersion) {
        for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
            collect_dependency_inlay_hints(
                document_tree,
                &[dependency_key],
                &cargo_toml_path,
                &mut cargo_lock,
                toml_version,
                visible_range,
                &mut hints,
            );
        }

        collect_dependency_inlay_hints(
            document_tree,
            &["workspace", "dependencies"],
            &cargo_toml_path,
            &mut cargo_lock,
            toml_version,
            visible_range,
            &mut hints,
        );

        if let Some((_, Value::Table(targets))) = dig_keys(document_tree, &["target"]) {
            for (target_key, target_value) in targets.key_values() {
                let Value::Table(_) = target_value else {
                    continue;
                };

                for dependency_key in ["dependencies", "dev-dependencies", "build-dependencies"] {
                    collect_dependency_inlay_hints(
                        document_tree,
                        &["target", target_key.value.as_str(), dependency_key],
                        &cargo_toml_path,
                        &mut cargo_lock,
                        toml_version,
                        visible_range,
                        &mut hints,
                    );
                }
            }
        }
    }

    if hints.is_empty() {
        Ok(None)
    } else {
        hints.sort_by_key(|hint| hint.position);
        Ok(Some(hints))
    }
}

fn collect_workspace_package_inlay_hints(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
    visible_range: tombi_text::Range,
    hints: &mut Vec<InlayHint>,
) {
    let mut resolved_workspace_document_tree = None;

    for package_item in WORKSPACE_PACKAGE_ITEMS {
        let Some((_, Value::Boolean(workspace))) =
            dig_keys(document_tree, &["package", package_item, "workspace"])
        else {
            continue;
        };
        if !workspace.value()
            || !tombi_text::Range::at(workspace.range().end).intersects(visible_range)
        {
            continue;
        }

        if resolved_workspace_document_tree.is_none() {
            resolved_workspace_document_tree =
                workspace_document_tree(document_tree, cargo_toml_path, toml_version);
        }
        let Some(workspace_document_tree) = resolved_workspace_document_tree
            .as_ref()
            .map(WorkspaceDocumentTree::as_tree)
        else {
            return;
        };

        let Some((_, workspace_value)) = dig_keys(
            workspace_document_tree,
            &["workspace", "package", package_item],
        ) else {
            continue;
        };

        let Some(label) = workspace_value_hint_label(workspace_value) else {
            continue;
        };

        hints.push(InlayHint {
            position: workspace.range().end,
            label,
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(WORKSPACE_PACKAGE_INHERITED_VALUE_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        });
    }
}

fn collect_dependency_inlay_hints(
    document_tree: &tombi_document_tree::DocumentTree,
    dependency_keys: &[&str],
    cargo_toml_path: &Path,
    cargo_lock: &mut Option<CargoLock>,
    toml_version: TomlVersion,
    visible_range: tombi_text::Range,
    hints: &mut Vec<InlayHint>,
) {
    let Some((_, Value::Table(dependencies))) = dig_keys(document_tree, dependency_keys) else {
        return;
    };

    let mut version_hints = Vec::new();

    for (dependency_key, dependency_value) in dependencies.key_values() {
        let Some(version_hint) = dependency_version_hint(
            dependency_package_name(&dependency_key.value, dependency_value),
            dependency_value,
        ) else {
            continue;
        };

        if !tombi_text::Range::at(version_hint.position).intersects(visible_range) {
            continue;
        }

        version_hints.push(version_hint);
    }

    if version_hints.is_empty() {
        return;
    }

    if cargo_lock.is_none() {
        *cargo_lock = load_cargo_lock(cargo_toml_path, toml_version);
    }

    let Some(cargo_lock) = cargo_lock.as_ref() else {
        return;
    };

    let current_package = if dependency_keys == ["workspace", "dependencies"] {
        None
    } else {
        current_package(document_tree, cargo_toml_path, toml_version)
    };
    let workspace_member_packages = if dependency_keys == ["workspace", "dependencies"] {
        workspace_member_packages(document_tree, cargo_toml_path, toml_version)
    } else {
        None
    };

    let workspace_resolved_versions = if dependency_keys == ["workspace", "dependencies"] {
        workspace_dependency_lock_versions(
            cargo_lock,
            workspace_member_packages.as_deref(),
            version_hints
                .iter()
                .map(|version_hint| version_hint.dependency_name.as_str()),
        )
    } else {
        BTreeMap::new()
    };

    for version_hint in version_hints {
        let resolved_version = if dependency_keys == ["workspace", "dependencies"] {
            let Some(resolved_version) =
                workspace_resolved_versions.get(&version_hint.dependency_name)
            else {
                continue;
            };
            resolved_version.clone()
        } else {
            let Some(resolved_version) = cargo_lock_dependency_version(
                cargo_lock,
                dependency_keys,
                &version_hint.dependency_name,
                current_package.as_ref(),
                workspace_member_packages.as_deref(),
            ) else {
                continue;
            };

            resolved_version
        };

        let Some(label) = version_hint_label(
            version_hint.current_version.as_deref(),
            &resolved_version,
            version_hint.always_show,
        ) else {
            continue;
        };

        hints.push(InlayHint {
            position: version_hint.position,
            label,
            kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
            tooltip: Some(RESOLVED_VERSION_TOOLTIP.to_string()),
            padding_left: Some(true),
            padding_right: Some(false),
        });
    }
}

fn dependency_version_hint(
    dependency_name: &str,
    dependency_value: &Value,
) -> Option<DependencyVersionHint> {
    match dependency_value {
        Value::String(version) => Some(DependencyVersionHint {
            dependency_name: dependency_name.to_string(),
            position: version.range().end,
            current_version: Some(version.value().to_string()),
            always_show: false,
        }),
        Value::Table(table) => {
            if let Some(Value::String(version)) = table.get("version") {
                return Some(DependencyVersionHint {
                    dependency_name: dependency_name.to_string(),
                    position: version.range().end,
                    current_version: Some(version.value().to_string()),
                    always_show: false,
                });
            }

            if let Some(Value::Boolean(workspace)) = table.get("workspace")
                && workspace.value()
            {
                return Some(DependencyVersionHint {
                    dependency_name: dependency_name.to_string(),
                    position: workspace.range().end,
                    current_version: None,
                    always_show: true,
                });
            }

            if let Some(Value::String(path)) = table.get("path") {
                return Some(DependencyVersionHint {
                    dependency_name: dependency_name.to_string(),
                    position: path.range().end,
                    current_version: None,
                    always_show: false,
                });
            }

            if let Some(Value::String(git)) = table.get("git") {
                return Some(DependencyVersionHint {
                    dependency_name: dependency_name.to_string(),
                    position: git.range().end,
                    current_version: None,
                    always_show: false,
                });
            }

            None
        }
        _ => None,
    }
}

fn cargo_lock_dependency_version(
    cargo_lock: &CargoLock,
    dependency_keys: &[&str],
    dependency_name: &str,
    current_package: Option<&CurrentPackage<'_>>,
    workspace_member_packages: Option<&[WorkspaceMemberPackage]>,
) -> Option<String> {
    if dependency_keys == ["workspace", "dependencies"] {
        return workspace_dependency_lock_version(
            cargo_lock,
            workspace_member_packages,
            dependency_name,
        );
    }

    let current_package = current_package?;
    cargo_lock.dependency_version_for_package(
        current_package.name,
        &current_package.version,
        dependency_name,
    )
}

fn workspace_dependency_lock_version(
    cargo_lock: &CargoLock,
    workspace_member_packages: Option<&[WorkspaceMemberPackage]>,
    dependency_name: &str,
) -> Option<String> {
    let workspace_member_packages = workspace_member_packages?;

    let resolved_versions = workspace_member_packages
        .iter()
        .filter_map(|package| {
            cargo_lock.dependency_version_for_package(
                &package.name,
                &package.version,
                dependency_name,
            )
        })
        .collect::<BTreeSet<_>>();

    (resolved_versions.len() == 1)
        .then(|| resolved_versions.into_iter().next())
        .flatten()
}

fn workspace_dependency_lock_versions<'a>(
    cargo_lock: &CargoLock,
    workspace_member_packages: Option<&[WorkspaceMemberPackage]>,
    dependency_names: impl Iterator<Item = &'a str>,
) -> BTreeMap<String, String> {
    let Some(workspace_member_packages) = workspace_member_packages else {
        return BTreeMap::new();
    };

    let dependency_names = dependency_names
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if dependency_names.is_empty() {
        return BTreeMap::new();
    }

    let mut resolved_versions = dependency_names
        .into_iter()
        .map(|dependency_name| (dependency_name, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();

    for package in workspace_member_packages {
        let Some(lock_package) = cargo_lock.package(&package.name, &package.version) else {
            continue;
        };

        for dependency in &lock_package.dependencies {
            let Some(versions) = resolved_versions.get_mut(&dependency.name) else {
                continue;
            };
            let Some(resolved_version) = cargo_lock.resolved_dependency_version(dependency) else {
                continue;
            };

            versions.insert(resolved_version);
        }
    }

    resolved_versions
        .into_iter()
        .filter_map(|(dependency_name, versions)| {
            (versions.len() == 1)
                .then(|| {
                    versions
                        .into_iter()
                        .next()
                        .map(|version| (dependency_name, version))
                })
                .flatten()
        })
        .collect()
}

fn workspace_member_packages(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<Vec<WorkspaceMemberPackage>> {
    if document_tree.contains_key("workspace") {
        return workspace_member_packages_for_workspace(
            document_tree,
            cargo_toml_path,
            toml_version,
        );
    }

    let (workspace_cargo_toml_path, _, workspace_document_tree) = find_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_path(document_tree),
        toml_version,
    )?;

    workspace_member_packages_for_workspace(
        &workspace_document_tree,
        &workspace_cargo_toml_path,
        toml_version,
    )
}

fn workspace_document_tree<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<WorkspaceDocumentTree<'a>> {
    if document_tree.contains_key("workspace") {
        return Some(WorkspaceDocumentTree::Current(document_tree));
    }

    let (_, _, workspace_document_tree) = find_workspace_cargo_toml(
        cargo_toml_path,
        get_workspace_path(document_tree),
        toml_version,
    )?;

    Some(WorkspaceDocumentTree::External(workspace_document_tree))
}

fn workspace_member_packages_for_workspace(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace_cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<Vec<WorkspaceMemberPackage>> {
    let member_patterns = workspace_member_patterns(workspace_document_tree);
    if member_patterns.is_empty() {
        return None;
    }

    let exclude_patterns = extract_exclude_patterns(workspace_document_tree);
    let workspace_dir_path = workspace_cargo_toml_path.parent()?;
    Some(
        find_package_cargo_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
            .filter_map(|(_, manifest_path)| {
                let manifest_path = manifest_path.canonicalize().unwrap_or(manifest_path);
                let (_, member_document_tree) = load_cargo_toml(&manifest_path, toml_version)?;
                let package_name = current_package_name(&member_document_tree)?;
                let package_version =
                    package_version(&member_document_tree, &manifest_path, toml_version)?;

                Some(WorkspaceMemberPackage {
                    name: package_name.to_string(),
                    version: package_version,
                })
            })
            .collect(),
    )
}

fn current_package<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<CurrentPackage<'a>> {
    Some(CurrentPackage {
        name: current_package_name(document_tree)?,
        version: package_version(document_tree, cargo_toml_path, toml_version)?,
    })
}

fn workspace_member_patterns(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
) -> Vec<&tombi_document_tree::String> {
    match dig_keys(workspace_document_tree, &["workspace", "members"]) {
        Some((_, Value::Array(members))) => members
            .iter()
            .filter_map(|member| match member {
                Value::String(pattern) => Some(pattern),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn load_cargo_lock(cargo_toml_path: &Path, toml_version: TomlVersion) -> Option<CargoLock> {
    let cargo_lock_path = find_cargo_lock_path(cargo_toml_path)?;
    let cargo_lock_text = std::fs::read_to_string(cargo_lock_path).ok()?;
    let root = tombi_ast::Root::cast(tombi_parser::parse(&cargo_lock_text).into_syntax_node())?;
    let document_tree = root.try_into_document_tree(toml_version).ok()?;

    CargoLock::from_document_tree(&document_tree)
}

fn find_cargo_lock_path(cargo_toml_path: &Path) -> Option<std::path::PathBuf> {
    let mut current_dir = cargo_toml_path.parent()?;

    loop {
        let candidate = current_dir.join("Cargo.lock");
        if candidate.is_file() {
            return candidate.canonicalize().ok().or(Some(candidate));
        }

        current_dir = current_dir.parent()?;
    }
}

fn package_version(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<String> {
    let (_, package_version) = dig_keys(document_tree, &["package", "version"])?;

    match package_version {
        Value::String(version) => Some(version.value().to_string()),
        Value::Table(table) => {
            let Some(Value::Boolean(workspace)) = table.get("workspace") else {
                return None;
            };
            if !workspace.value() {
                return None;
            }

            if document_tree.contains_key("workspace") {
                let (_, Value::String(version)) =
                    dig_keys(document_tree, &["workspace", "package", "version"])?
                else {
                    return None;
                };

                return Some(version.value().to_string());
            }

            let (_, _, workspace_document_tree) = find_workspace_cargo_toml(
                cargo_toml_path,
                get_workspace_path(document_tree),
                toml_version,
            )?;
            let (_, Value::String(version)) = dig_keys(
                &workspace_document_tree,
                &["workspace", "package", "version"],
            )?
            else {
                return None;
            };

            Some(version.value().to_string())
        }
        _ => None,
    }
}

fn current_package_name(document_tree: &tombi_document_tree::DocumentTree) -> Option<&str> {
    let (_, Value::String(package_name)) = dig_keys(document_tree, &["package", "name"])? else {
        return None;
    };

    Some(package_name.value())
}

fn version_hint_label(
    current_version: Option<&str>,
    resolved_version: &str,
    always_show: bool,
) -> Option<String> {
    if !always_show && current_version == Some(resolved_version) {
        return None;
    }

    Some(format!(r#" → "{resolved_version}""#))
}

fn workspace_value_hint_label(value: &Value) -> Option<String> {
    if matches!(value, Value::Incomplete { .. }) {
        return None;
    }

    Some(format!(" → {}", sanitize_value_for_hint(value)))
}

fn sanitize_value_for_hint(value: &Value) -> String {
    let sanitized = value.to_string().replace('\r', "\\r").replace('\n', "\\n");
    let value_len = sanitized.chars().count();
    if value_len <= MAX_WORKSPACE_VALUE_HINT_CHARS {
        return sanitized;
    }

    let truncated_len = MAX_WORKSPACE_VALUE_HINT_CHARS.saturating_sub(3);
    let mut truncated = sanitized.chars().take(truncated_len).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn cargo_inlay_hint_root_enabled(features: Option<&tombi_config::CargoExtensionFeatures>) -> bool {
    features.map_or(
        true,
        tombi_config::CargoExtensionFeatures::inlay_hint_enabled,
    )
}

fn cargo_inlay_hint_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
    feature: CargoInlayHintFeature,
) -> bool {
    match feature {
        CargoInlayHintFeature::DependencyVersion => features.map_or(
            true,
            tombi_config::CargoExtensionFeatures::dependency_version_inlay_hint_enabled,
        ),
    }
}

impl CargoLock {
    fn from_document_tree(document_tree: &tombi_document_tree::DocumentTree) -> Option<Self> {
        let (_, Value::Array(packages)) = dig_keys(document_tree, &["package"])? else {
            return None;
        };

        Some(Self {
            packages: packages
                .iter()
                .filter_map(CargoLockPackage::from_value)
                .collect(),
        })
    }

    fn package(&self, package_name: &str, package_version: &str) -> Option<&CargoLockPackage> {
        self.packages
            .iter()
            .find(|package| package.name == package_name && package.version == package_version)
    }

    fn resolved_dependency_version(&self, dependency: &CargoLockDependency) -> Option<String> {
        dependency
            .version
            .clone()
            .or_else(|| self.unique_package_version(&dependency.name))
    }

    fn unique_package_version(&self, dependency_name: &str) -> Option<String> {
        let package_versions = self
            .packages
            .iter()
            .filter(|package| package.name == dependency_name)
            .map(|package| package.version.clone())
            .collect::<BTreeSet<_>>();

        (package_versions.len() == 1)
            .then(|| package_versions.into_iter().next())
            .flatten()
    }

    fn dependency_version_for_package(
        &self,
        package_name: &str,
        package_version: &str,
        dependency_name: &str,
    ) -> Option<String> {
        let package = self.package(package_name, package_version)?;

        let explicit_versions = package
            .dependencies
            .iter()
            .filter(|dependency| dependency.name == dependency_name)
            .filter_map(|dependency| dependency.version.clone())
            .collect::<BTreeSet<_>>();

        if explicit_versions.len() == 1 {
            return explicit_versions.into_iter().next();
        }

        if explicit_versions.len() > 1 {
            return None;
        }

        self.unique_package_version(dependency_name)
    }
}

impl CargoLockPackage {
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
        let dependencies = match table.get("dependencies") {
            Some(Value::Array(dependencies)) => dependencies
                .iter()
                .filter_map(CargoLockDependency::from_value)
                .collect(),
            _ => Vec::new(),
        };

        Some(Self {
            name,
            version,
            dependencies,
        })
    }
}

impl CargoLockDependency {
    fn from_value(value: &Value) -> Option<Self> {
        let Value::String(dependency) = value else {
            return None;
        };

        Some(Self::parse(dependency.value()))
    }

    fn parse(value: &str) -> Self {
        if let Some((name, version)) = value.rsplit_once(' ')
            && version.as_bytes().first().is_some_and(u8::is_ascii_digit)
        {
            return Self {
                name: name.to_string(),
                version: Some(version.to_string()),
            };
        }

        Self {
            name: value.to_string(),
            version: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn parse_value(source: &str) -> Value {
        let root = tombi_ast::Root::cast(tombi_parser::parse(source).into_syntax_node())
            .expect("expected root");
        let document_tree = root
            .try_into_document_tree(TomlVersion::default())
            .expect("expected document tree");
        let (_, value) = dig_keys(&document_tree, &["value"]).expect("expected value");
        value.clone()
    }

    #[test]
    fn adds_plain_version_label() {
        assert_eq!(
            version_hint_label(Some("0.15.6"), "0.15.8", false),
            Some(r#" → "0.15.8""#.to_string())
        );
    }

    #[test]
    fn omits_hint_when_version_is_already_resolved() {
        assert_eq!(version_hint_label(Some("0.15.8"), "0.15.8", false), None);
    }

    #[test]
    fn keeps_plain_version_when_current_version_is_missing() {
        assert_eq!(
            version_hint_label(None, "0.15.8", false),
            Some(r#" → "0.15.8""#.to_string())
        );
    }

    #[test]
    fn keeps_hint_for_workspace_inheritance_even_when_versions_match() {
        assert_eq!(
            version_hint_label(Some("0.15.8"), "0.15.8", true),
            Some(r#" → "0.15.8""#.to_string())
        );
    }

    #[test]
    fn renders_workspace_value_hint_label() {
        let value = parse_value(r#"value = ["tombi", "cargo"]"#);

        assert_eq!(
            workspace_value_hint_label(&value),
            Some(r#" → ["tombi", "cargo"]"#.to_string())
        );
    }

    #[test]
    fn normalizes_workspace_value_hint_label_to_single_line() {
        let value = parse_value(
            r#"
            value = """
            hello
            world
            """
            "#,
        );

        let label = workspace_value_hint_label(&value).expect("expected label");
        assert!(!label.contains('\n'));
        assert!(label.contains("\\n"));
    }

    #[test]
    fn truncates_workspace_value_hint_label() {
        let value = parse_value(
            r#"value = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz""#,
        );

        let label = workspace_value_hint_label(&value).expect("expected label");
        assert!(label.ends_with("..."));
        assert!(label.chars().count() <= MAX_WORKSPACE_VALUE_HINT_CHARS + 3);
    }

    #[test]
    fn keeps_workspace_package_hints_when_dependency_version_hints_are_disabled() {
        let temp_dir = tempfile::tempdir().expect("expected temp dir");
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        fs::write(
            &cargo_toml_path,
            r#"
            [package]
            name = "app"
            version = { workspace = true }

            [workspace]
            members = ["."]

            [workspace.package]
            version = "0.0.0-dev"
            "#,
        )
        .expect("expected Cargo.toml");

        let (_, document_tree) =
            load_cargo_toml(&cargo_toml_path, TomlVersion::default()).expect("expected manifest");
        let uri = tombi_uri::Uri::from_file_path(&cargo_toml_path).expect("expected uri");
        let features = tombi_config::CargoExtensionFeatures::Features(
            tombi_config::CargoExtensionFeatureTree {
                lsp: Some(tombi_config::CargoLspFeatures::Features(
                    tombi_config::CargoLspFeatureTree {
                        inlay_hint: Some(tombi_config::CargoInlayHintFeatures::Features(
                            tombi_config::CargoInlayHintFeatureTree {
                                dependency_version: Some(tombi_config::ToggleFeature {
                                    enabled: Some(false.into()),
                                }),
                            },
                        )),
                        ..Default::default()
                    },
                )),
            },
        );

        let hints = inlay_hint_impl(
            &uri,
            &document_tree,
            tombi_text::Range::new(
                tombi_text::Position::new(0, 0),
                tombi_text::Position::new(8, 0),
            ),
            TomlVersion::default(),
            Some(&features),
        )
        .expect("expected inlay hint result");

        assert_eq!(
            hints,
            Some(vec![InlayHint {
                position: tombi_text::Position::new(3, 40),
                label: r#" → "0.0.0-dev""#.to_string(),
                kind: Some(tower_lsp::lsp_types::InlayHintKind::TYPE),
                tooltip: Some(WORKSPACE_PACKAGE_INHERITED_VALUE_TOOLTIP.to_string()),
                padding_left: Some(true),
                padding_right: Some(false),
            }])
        );
    }

    #[test]
    fn parses_lockfile_dependency_with_explicit_version() {
        let dependency = CargoLockDependency::parse("windows-sys 0.59.0");

        assert_eq!(dependency.name, "windows-sys");
        assert_eq!(dependency.version.as_deref(), Some("0.59.0"));
    }

    #[test]
    fn parses_lockfile_dependency_without_explicit_version() {
        let dependency = CargoLockDependency::parse("serde");

        assert_eq!(dependency.name, "serde");
        assert_eq!(dependency.version, None);
    }
}
