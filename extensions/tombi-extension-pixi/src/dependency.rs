use tombi_document_tree_syntax::{DocumentTree, Value, dig_accessors};
use tombi_schema_store::Accessor;

const DEPENDENCY_TABLES: &[&str] = &["dependencies", "host-dependencies", "build-dependencies"];
const PACKAGE_DEPENDENCY_TABLES: &[&str] = &[
    "run-dependencies",
    "host-dependencies",
    "build-dependencies",
];

/// A conda dependency entry in `pixi.toml`, e.g. `[feature.test.target.linux-64.dependencies] pytest = "*"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CondaDependency<'a> {
    /// Accessors up to and including the package name key.
    pub accessors: &'a [Accessor],
    pub package_name: &'a str,
    pub feature: Option<&'a str>,
    /// The `target.<selector>` the dependency is declared under.
    pub target: Option<&'a str>,
    /// Whether the accessors point at the `version` key of a table dependency.
    pub is_version_key: bool,
}

/// Matches accessors pointing at a conda dependency or its `version` key.
pub(crate) fn find_conda_dependency(accessors: &[Accessor]) -> Option<CondaDependency<'_>> {
    let mut index = 0;
    let mut feature = None;
    let mut target = None;

    if accessors.first()?.as_key()? == "feature" {
        feature = Some(accessors.get(1)?.as_key()?);
        index = 2;
    }

    let tables = if feature.is_none() && accessors.get(index)?.as_key()? == "package" {
        index += 1;
        PACKAGE_DEPENDENCY_TABLES
    } else {
        DEPENDENCY_TABLES
    };

    if accessors.get(index)?.as_key()? == "target" {
        target = Some(accessors.get(index + 1)?.as_key()?);
        index += 2;
    }

    if !tables.contains(&accessors.get(index)?.as_key()?) {
        return None;
    }

    let package_name = accessors.get(index + 1)?.as_key()?;
    let rest = &accessors[index + 2..];
    let is_version_key = match rest {
        [] => false,
        [Accessor::Key(key)] if key == "version" => true,
        _ => return None,
    };

    Some(CondaDependency {
        accessors: &accessors[..index + 2],
        package_name,
        feature,
        target,
        is_version_key,
    })
}

/// Returns `None` for source dependencies (`path`, `git`, `url`), which are not resolved from channels.
/// Otherwise returns the dependency's inline `channel`, if any.
pub(crate) fn channel_dependency_source<'a>(
    document_tree: &'a DocumentTree<'a>,
    dependency: &CondaDependency<'_>,
) -> Option<Option<&'a str>> {
    match dig_accessors(document_tree, dependency.accessors) {
        Some((_, Value::Table(table))) => {
            if ["path", "git", "url"]
                .iter()
                .any(|key| table.contains_key(key))
            {
                return None;
            }
            match table.get("channel") {
                Some(Value::String(channel)) => Some(Some(channel.value())),
                _ => Some(None),
            }
        }
        _ => Some(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(keys: &[&str]) -> Vec<Accessor> {
        keys.iter()
            .map(|key| Accessor::Key(key.to_string()))
            .collect()
    }

    #[test]
    fn matches_root_dependency() {
        let accessors = keys(&["dependencies", "python"]);
        let dependency = find_conda_dependency(&accessors).unwrap();
        assert_eq!(dependency.package_name, "python");
        assert_eq!(dependency.accessors.len(), 2);
        assert!(!dependency.is_version_key);
    }

    #[test]
    fn matches_version_key() {
        let accessors = keys(&["host-dependencies", "python", "version"]);
        let dependency = find_conda_dependency(&accessors).unwrap();
        assert_eq!(dependency.package_name, "python");
        assert_eq!(dependency.accessors.len(), 2);
        assert!(dependency.is_version_key);
    }

    #[test]
    fn matches_feature_target_dependency() {
        let accessors = keys(&[
            "feature",
            "test",
            "target",
            "linux-64",
            "build-dependencies",
            "pytest",
        ]);
        let dependency = find_conda_dependency(&accessors).unwrap();
        assert_eq!(dependency.package_name, "pytest");
        assert_eq!(dependency.feature, Some("test"));
        assert_eq!(dependency.target, Some("linux-64"));
    }

    #[test]
    fn matches_package_run_dependency() {
        let accessors = keys(&["package", "run-dependencies", "numpy"]);
        assert_eq!(
            find_conda_dependency(&accessors).map(|dependency| dependency.package_name),
            Some("numpy")
        );
    }

    #[test]
    fn rejects_non_conda_dependencies() {
        assert_eq!(
            find_conda_dependency(&keys(&["pypi-dependencies", "requests"])),
            None
        );
        assert_eq!(find_conda_dependency(&keys(&["dependencies"])), None);
        assert_eq!(
            find_conda_dependency(&keys(&["dependencies", "python", "build"])),
            None
        );
        assert_eq!(
            find_conda_dependency(&keys(&["package", "dependencies", "python"])),
            None
        );
        assert_eq!(
            find_conda_dependency(&keys(&["workspace", "channels"])),
            None
        );
    }
}
