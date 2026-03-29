use std::{collections::BTreeSet, path::Path};

use tombi_ast::AstNode;
use tombi_config::TomlVersion;
use tombi_document_tree::{TryIntoDocumentTree, Value, dig_keys};

#[derive(Debug, Clone)]
pub(crate) struct CargoLock {
    pub(crate) packages: Vec<CargoLockPackage>,
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

pub(crate) fn load_cargo_lock(
    cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Option<CargoLock> {
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

    pub(crate) fn package(
        &self,
        package_name: &str,
        package_version: &str,
    ) -> Option<&CargoLockPackage> {
        self.packages
            .iter()
            .find(|package| package.name == package_name && package.version == package_version)
    }

    pub(crate) fn resolved_dependency_version(
        &self,
        dependency: &CargoLockDependency,
    ) -> Option<String> {
        dependency
            .version
            .clone()
            .or_else(|| self.unique_package_version(&dependency.name))
    }

    pub(crate) fn dependency_version_for_package(
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
}
