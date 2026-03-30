use std::path::Path;

use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{TryIntoDocumentTree, Value, dig_keys};
use tombi_hashmap::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub(crate) struct CargoLock {
    pub(crate) packages: Vec<CargoLockPackage>,
    unique_package_versions: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone)]
pub(crate) struct CargoLockPackage {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) dependencies: Vec<CargoLockDependency>,
}

#[derive(Debug, Clone)]
pub(crate) struct CargoLockDependency {
    pub(crate) name: String,
    pub(crate) version: Option<String>,
}

pub(crate) fn load_cargo_lock_from_path(
    cargo_lock_path: &Path,
    toml_version: TomlVersion,
) -> Option<CargoLock> {
    let cargo_lock_text = std::fs::read_to_string(cargo_lock_path).ok()?;
    let root = tombi_ast::Root::cast(tombi_parser::parse(&cargo_lock_text).into_syntax_node())?;
    let document_tree = root.try_into_document_tree(toml_version).ok()?;

    CargoLock::from_document_tree(&document_tree)
}

pub(crate) fn find_cargo_lock_path(cargo_toml_path: &Path) -> Option<std::path::PathBuf> {
    let mut current_dir = cargo_toml_path.parent()?;

    loop {
        let candidate = current_dir.join("Cargo.lock");
        if candidate.is_file() {
            return candidate.canonicalize().ok().or(Some(candidate));
        }

        current_dir = current_dir.parent()?;
    }
}

impl CargoLock {
    pub(crate) fn new(packages: Vec<CargoLockPackage>) -> Self {
        let unique_package_versions = compute_unique_package_versions(&packages);
        Self {
            packages,
            unique_package_versions,
        }
    }

    fn from_document_tree(document_tree: &tombi_document_tree::DocumentTree) -> Option<Self> {
        let (_, Value::Array(packages)) = dig_keys(document_tree, &["package"])? else {
            return None;
        };

        let packages = packages
            .iter()
            .filter_map(CargoLockPackage::from_value)
            .collect();
        Some(Self::new(packages))
    }

    pub(crate) fn unique_package_versions(&self) -> &HashMap<String, Option<String>> {
        &self.unique_package_versions
    }

    pub(crate) fn resolve_dependency_version(
        &self,
        crate_name: &str,
        version_requirement: &str,
    ) -> Option<String> {
        exact_crates_io_version(version_requirement)
            .or_else(|| self.unique_dependency_version(crate_name))
    }

    pub(crate) fn unique_dependency_version(&self, dependency_name: &str) -> Option<String> {
        let resolved_versions = self
            .packages
            .iter()
            .filter_map(|package| {
                package.lockfile_resolved_dependency_version(
                    dependency_name,
                    &self.unique_package_versions,
                )
            })
            .collect::<HashSet<_>>();

        (resolved_versions.len() == 1)
            .then(|| resolved_versions.into_iter().next())
            .flatten()
    }
}

pub(crate) fn exact_crates_io_version(version_requirement: &str) -> Option<String> {
    let version_requirement = version_requirement.trim();
    let version_requirement = version_requirement
        .strip_prefix('=')
        .map(str::trim)
        .unwrap_or(version_requirement);

    semver::Version::parse(version_requirement)
        .ok()
        .map(|version| version.to_string())
}

fn compute_unique_package_versions(
    packages: &[CargoLockPackage],
) -> HashMap<String, Option<String>> {
    let mut package_versions = HashMap::<String, HashSet<String>>::new();

    for package in packages {
        package_versions
            .entry(package.name.clone())
            .or_default()
            .insert(package.version.clone());
    }

    package_versions
        .into_iter()
        .map(|(crate_name, versions)| {
            let version = (versions.len() == 1)
                .then(|| versions.into_iter().next())
                .flatten();
            (crate_name, version)
        })
        .collect()
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

impl CargoLockPackage {
    pub(crate) fn lockfile_resolved_dependency_version(
        &self,
        dependency_name: &str,
        unique_package_versions: &HashMap<String, Option<String>>,
    ) -> Option<String> {
        let explicit_versions = self
            .dependencies
            .iter()
            .filter(|dependency| dependency.name == dependency_name)
            .filter_map(|dependency| dependency.version.clone())
            .collect::<HashSet<_>>();

        if explicit_versions.len() == 1 {
            return explicit_versions.into_iter().next();
        }

        if explicit_versions.len() > 1 {
            return None;
        }

        self.dependencies
            .iter()
            .any(|dependency| dependency.name == dependency_name)
            .then(|| {
                unique_package_versions
                    .get(dependency_name)
                    .cloned()
                    .flatten()
            })
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn resolves_unique_dependency_version_from_lockfile() {
        let cargo_lock = CargoLock::new(vec![
            CargoLockPackage {
                name: "demo".to_string(),
                version: "0.1.0".to_string(),
                dependencies: vec![CargoLockDependency {
                    name: "criterion".to_string(),
                    version: Some("0.5.1".to_string()),
                }],
            },
            CargoLockPackage {
                name: "helper".to_string(),
                version: "0.1.0".to_string(),
                dependencies: vec![CargoLockDependency {
                    name: "criterion".to_string(),
                    version: None,
                }],
            },
            CargoLockPackage {
                name: "criterion".to_string(),
                version: "0.5.1".to_string(),
                dependencies: Vec::new(),
            },
        ]);

        assert_eq!(
            cargo_lock.unique_dependency_version("criterion"),
            Some("0.5.1".to_string())
        );
    }

    #[test]
    fn does_not_resolve_ambiguous_dependency_version_from_lockfile() {
        let cargo_lock = CargoLock::new(vec![
            CargoLockPackage {
                name: "demo".to_string(),
                version: "0.1.0".to_string(),
                dependencies: vec![CargoLockDependency {
                    name: "faststr".to_string(),
                    version: Some("0.2.1".to_string()),
                }],
            },
            CargoLockPackage {
                name: "helper".to_string(),
                version: "0.1.0".to_string(),
                dependencies: vec![CargoLockDependency {
                    name: "faststr".to_string(),
                    version: Some("0.2.2".to_string()),
                }],
            },
        ]);

        assert_eq!(cargo_lock.unique_dependency_version("faststr"), None);
    }
}
