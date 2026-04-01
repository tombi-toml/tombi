use std::path::{Path, PathBuf};

use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_document_tree::{Value, dig_accessors, dig_keys};
use tombi_schema_store::{Accessor, matches_accessors};

use crate::{
    canonicalize_or_original, find_cargo_toml, find_package_cargo_toml_paths,
    find_workspace_cargo_toml, get_workspace_cargo_toml_path, load_cargo_toml,
    workspace::{extract_exclude_patterns, extract_member_patterns},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CargoFeatureRef<'a> {
    LocalFeature(&'a str),
    OptionalDependency(&'a str),
    DependencyFeature {
        dep_key: &'a str,
        feature: &'a str,
        weak: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CargoTargetLocation {
    pub(crate) cargo_toml_path: PathBuf,
    pub(crate) range: tombi_text::Range,
}

impl CargoTargetLocation {
    pub(crate) fn definition_location(&self) -> Option<tombi_extension::DefinitionLocation> {
        let uri = tombi_uri::Uri::from_file_path(&self.cargo_toml_path).ok()?;
        Some(tombi_extension::DefinitionLocation {
            uri,
            range: self.range,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CargoFeatureUsageTarget {
    LocalFeature {
        cargo_toml_path: PathBuf,
        feature_name: String,
    },
    OptionalDependency {
        cargo_toml_path: PathBuf,
        dep_key: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResolvedCargoFeatureTarget {
    LocalFeature {
        location: CargoTargetLocation,
        feature_name: String,
    },
    OptionalDependency {
        location: CargoTargetLocation,
        dep_key: String,
    },
}

impl ResolvedCargoFeatureTarget {
    fn into_location(self) -> CargoTargetLocation {
        match self {
            Self::LocalFeature { location, .. } | Self::OptionalDependency { location, .. } => {
                location
            }
        }
    }

    fn matches_usage_target(&self, target: &CargoFeatureUsageTarget) -> bool {
        match (self, target) {
            (
                Self::LocalFeature {
                    location,
                    feature_name,
                },
                CargoFeatureUsageTarget::LocalFeature {
                    cargo_toml_path,
                    feature_name: target_feature_name,
                },
            ) => {
                location.cargo_toml_path == *cargo_toml_path && feature_name == target_feature_name
            }
            (
                Self::OptionalDependency { location, dep_key },
                CargoFeatureUsageTarget::OptionalDependency {
                    cargo_toml_path,
                    dep_key: target_dep_key,
                },
            ) => location.cargo_toml_path == *cargo_toml_path && dep_key == target_dep_key,
            _ => false,
        }
    }
}

pub(crate) fn parse_cargo_feature_ref(value: &str) -> CargoFeatureRef<'_> {
    if let Some(name) = value.strip_prefix("dep:") {
        return CargoFeatureRef::OptionalDependency(name);
    }

    if let Some((dep_key, feature)) = value.split_once("?/") {
        return CargoFeatureRef::DependencyFeature {
            dep_key,
            feature,
            weak: true,
        };
    }

    if let Some((dep_key, feature)) = value.split_once('/') {
        return CargoFeatureRef::DependencyFeature {
            dep_key,
            feature,
            weak: false,
        };
    }

    CargoFeatureRef::LocalFeature(value)
}

pub(crate) fn resolve_feature_table_string(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    feature_string: &tombi_document_tree::String,
    toml_version: TomlVersion,
) -> Option<CargoTargetLocation> {
    resolve_feature_table_string_target(
        document_tree,
        cargo_toml_path,
        feature_string,
        toml_version,
    )
    .map(ResolvedCargoFeatureTarget::into_location)
}

pub(crate) fn resolve_dependency_feature_string(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    dependency_accessors: &[Accessor],
    feature_string: &tombi_document_tree::String,
    toml_version: TomlVersion,
) -> Option<CargoTargetLocation> {
    resolve_dependency_feature_string_target(
        document_tree,
        cargo_toml_path,
        dependency_accessors,
        feature_string,
        toml_version,
    )
    .map(ResolvedCargoFeatureTarget::into_location)
}

pub(crate) fn find_local_feature(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    feature_name: &str,
) -> Option<CargoTargetLocation> {
    let (feature_key, _) = dig_keys(document_tree, &["features", feature_name])?;
    Some(CargoTargetLocation {
        cargo_toml_path: canonicalize_or_original(cargo_toml_path.to_path_buf()),
        range: feature_key.unquoted_range(),
    })
}

pub(crate) fn find_optional_dependency(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    dep_key: &str,
) -> Option<CargoTargetLocation> {
    find_non_workspace_dependency_entry(document_tree, dep_key).and_then(|accessors| {
        let Some((_, Value::Table(table))) = dig_accessors(document_tree, accessors.as_slice())
        else {
            return None;
        };
        let optional_value = table.get("optional")?;
        match optional_value {
            Value::Boolean(optional) if optional.value() => Some(CargoTargetLocation {
                cargo_toml_path: canonicalize_or_original(cargo_toml_path.to_path_buf()),
                range: optional.range(),
            }),
            _ => None,
        }
    })
}

pub(crate) fn collect_feature_usage_locations(
    current_document_tree: &tombi_document_tree::DocumentTree,
    current_cargo_toml_path: &Path,
    target: &CargoFeatureUsageTarget,
    toml_version: TomlVersion,
) -> Vec<CargoTargetLocation> {
    let current_canonical = canonicalize_or_original(current_cargo_toml_path.to_path_buf());
    let mut locations = collect_feature_usage_locations_in_manifest(
        current_document_tree,
        &current_canonical,
        target,
        toml_version,
    );
    let manifest_paths =
        workspace_manifest_paths(current_document_tree, current_cargo_toml_path, toml_version);

    for manifest_path in manifest_paths.into_iter().map(canonicalize_or_original) {
        if manifest_path == current_canonical {
            continue;
        }

        let Some((_, document_tree)) = load_cargo_toml(&manifest_path, toml_version) else {
            continue;
        };

        locations.extend(collect_feature_usage_locations_in_manifest(
            &document_tree,
            &manifest_path,
            target,
            toml_version,
        ));
    }

    locations
        .into_iter()
        .unique_by(|location| (location.cargo_toml_path.clone(), location.range))
        .collect()
}

pub(crate) fn collect_feature_usage_locations_in_manifest(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    target: &CargoFeatureUsageTarget,
    toml_version: TomlVersion,
) -> Vec<CargoTargetLocation> {
    let mut locations =
        collect_feature_table_usage_locations(document_tree, cargo_toml_path, target, toml_version);
    locations.extend(collect_dependency_feature_usage_locations(
        document_tree,
        cargo_toml_path,
        target,
        toml_version,
    ));
    locations
}

pub(crate) fn feature_table_string_at_accessors<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    accessors: &'a [Accessor],
) -> Option<&'a tombi_document_tree::String> {
    if !matches_accessors!(accessors, ["features", _, _]) {
        return None;
    }
    let Some((_, Value::String(feature_string))) = dig_accessors(document_tree, accessors) else {
        return None;
    };
    Some(feature_string)
}

pub(crate) fn feature_usage_target_for_feature_key(
    cargo_toml_path: &Path,
    accessors: &[Accessor],
) -> Option<CargoFeatureUsageTarget> {
    if !matches_accessors!(accessors, ["features", _]) {
        return None;
    }

    let Accessor::Key(feature_name) = accessors.get(1)? else {
        return None;
    };

    Some(CargoFeatureUsageTarget::LocalFeature {
        cargo_toml_path: canonicalize_or_original(cargo_toml_path.to_path_buf()),
        feature_name: feature_name.clone(),
    })
}

pub(crate) fn dependency_feature_string_context<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    accessors: &'a [Accessor],
) -> Option<(&'a tombi_document_tree::String, Vec<Accessor>)> {
    let dependency_accessors =
        if matches_accessors!(accessors, ["workspace", "dependencies", _, "features", _]) {
            accessors[..3].to_vec()
        } else if matches_accessors!(accessors, ["dependencies", _, "features", _])
            || matches_accessors!(accessors, ["dev-dependencies", _, "features", _])
            || matches_accessors!(accessors, ["build-dependencies", _, "features", _])
        {
            accessors[..2].to_vec()
        } else if matches_accessors!(accessors, ["target", _, "dependencies", _, "features", _])
            || matches_accessors!(
                accessors,
                ["target", _, "dev-dependencies", _, "features", _]
            )
            || matches_accessors!(
                accessors,
                ["target", _, "build-dependencies", _, "features", _]
            )
        {
            accessors[..4].to_vec()
        } else {
            return None;
        };

    let Some((_, Value::String(feature_string))) = dig_accessors(document_tree, accessors) else {
        return None;
    };
    Some((feature_string, dependency_accessors))
}

pub(crate) fn feature_key_at_accessors<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    accessors: &[Accessor],
) -> Option<&'a tombi_document_tree::Key> {
    if !matches_accessors!(accessors, ["features", _]) {
        return None;
    }
    let Accessor::Key(feature_name) = accessors.get(1)? else {
        return None;
    };
    dig_keys(document_tree, &["features", feature_name.as_str()]).map(|(key, _)| key)
}

pub(crate) fn optional_dependency_value_at_accessors<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
    accessors: &'a [Accessor],
) -> Option<&'a tombi_document_tree::Boolean> {
    let dependency_accessors = dependency_optional_accessors(accessors)?;
    let Some((_, Value::Table(table))) = dig_accessors(document_tree, dependency_accessors) else {
        return None;
    };
    let Value::Boolean(optional) = table.get("optional")? else {
        return None;
    };
    Some(optional)
}

pub(crate) fn feature_usage_target_for_optional_dependency(
    cargo_toml_path: &Path,
    accessors: &[Accessor],
) -> Option<CargoFeatureUsageTarget> {
    let dependency_accessors = dependency_optional_accessors(accessors)?;
    let Accessor::Key(dep_key) = dependency_accessors.last()? else {
        return None;
    };

    Some(CargoFeatureUsageTarget::OptionalDependency {
        cargo_toml_path: canonicalize_or_original(cargo_toml_path.to_path_buf()),
        dep_key: dep_key.clone(),
    })
}

pub(crate) fn dependency_optional_accessors(accessors: &[Accessor]) -> Option<&[Accessor]> {
    if matches_accessors!(accessors, ["dependencies", _, "optional"])
        || matches_accessors!(accessors, ["dev-dependencies", _, "optional"])
        || matches_accessors!(accessors, ["build-dependencies", _, "optional"])
    {
        Some(&accessors[..2])
    } else if matches_accessors!(accessors, ["target", _, "dependencies", _, "optional"])
        || matches_accessors!(accessors, ["target", _, "dev-dependencies", _, "optional"])
        || matches_accessors!(
            accessors,
            ["target", _, "build-dependencies", _, "optional"]
        )
    {
        Some(&accessors[..4])
    } else {
        None
    }
}

struct ResolvedDependency {
    cargo_toml_path: PathBuf,
    document_tree: tombi_document_tree::DocumentTree,
}

fn resolve_named_dependency(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    dependency_accessors: &[Accessor],
    toml_version: TomlVersion,
) -> Option<ResolvedDependency> {
    let (_, dep_value) = dig_accessors(document_tree, dependency_accessors)?;
    let Value::Table(table) = dep_value else {
        return None;
    };

    if let Some(Value::String(crate_path)) = table.get("path") {
        return load_resolved_dependency(cargo_toml_path, crate_path.value(), toml_version);
    }

    if let Some(Value::Boolean(workspace)) = table.get("workspace")
        && workspace.value()
    {
        let (workspace_cargo_toml_path, _, workspace_document_tree) = find_workspace_cargo_toml(
            cargo_toml_path,
            get_workspace_cargo_toml_path(document_tree),
            toml_version,
        )?;
        let dependency_key = match dependency_accessors.last() {
            Some(Accessor::Key(dep_key)) => dep_key,
            _ => return None,
        };
        let (_, workspace_dep_value) = dig_keys(
            &workspace_document_tree,
            &["workspace", "dependencies", dependency_key.as_str()],
        )?;
        let Value::Table(workspace_table) = workspace_dep_value else {
            return None;
        };
        if let Some(Value::String(crate_path)) = workspace_table.get("path") {
            return load_resolved_dependency(
                &workspace_cargo_toml_path,
                crate_path.value(),
                toml_version,
            );
        }
    }

    None
}

fn load_resolved_dependency(
    cargo_toml_path: &Path,
    crate_path: &str,
    toml_version: TomlVersion,
) -> Option<ResolvedDependency> {
    let (resolved_path, _, document_tree) =
        find_cargo_toml(cargo_toml_path, Path::new(crate_path), toml_version)?;
    Some(ResolvedDependency {
        cargo_toml_path: resolved_path,
        document_tree,
    })
}

fn has_explicit_dep_feature(
    document_tree: &tombi_document_tree::DocumentTree,
    dep_key: &str,
) -> bool {
    let Some((_, Value::Table(features_table))) = dig_keys(document_tree, &["features"]) else {
        return false;
    };

    features_table.values().any(|value| match value {
        Value::Array(features) => {
            features
                .values()
                .iter()
                .any(|feature_value| match feature_value {
                    Value::String(feature_string) => matches!(
                        parse_cargo_feature_ref(feature_string.value()),
                        CargoFeatureRef::OptionalDependency(name) if name == dep_key
                    ),
                    _ => false,
                })
        }
        _ => false,
    })
}

fn find_non_workspace_dependency_entry(
    document_tree: &tombi_document_tree::DocumentTree,
    dep_key: &str,
) -> Option<Vec<Accessor>> {
    for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if dig_keys(document_tree, &[dependency_kind, dep_key]).is_some() {
            return Some(vec![
                Accessor::Key(dependency_kind.to_string()),
                Accessor::Key(dep_key.to_string()),
            ]);
        }
    }

    let Some((_, Value::Table(targets))) = dig_keys(document_tree, &["target"]) else {
        return None;
    };

    for (target_key, target_value) in targets.key_values() {
        let Value::Table(target_table) = target_value else {
            continue;
        };
        for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if target_table
                .get_key_value(dependency_kind)
                .and_then(|(_, value)| match value {
                    Value::Table(dependencies) => dependencies.get(dep_key),
                    _ => None,
                })
                .is_some()
            {
                return Some(vec![
                    Accessor::Key("target".to_string()),
                    Accessor::Key(target_key.value.to_string()),
                    Accessor::Key(dependency_kind.to_string()),
                    Accessor::Key(dep_key.to_string()),
                ]);
            }
        }
    }

    None
}

fn collect_feature_table_usage_locations(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    target: &CargoFeatureUsageTarget,
    toml_version: TomlVersion,
) -> Vec<CargoTargetLocation> {
    let Some((_, Value::Table(features_table))) = dig_keys(document_tree, &["features"]) else {
        return Vec::new();
    };

    features_table
        .values()
        .filter_map(|value| match value {
            Value::Array(features) => Some(features),
            _ => None,
        })
        .flat_map(|features| features.values())
        .filter_map(|value| match value {
            Value::String(feature_string) => Some(feature_string),
            _ => None,
        })
        .filter_map(|feature_string| {
            let resolved = resolve_feature_table_string_target(
                document_tree,
                cargo_toml_path,
                feature_string,
                toml_version,
            )?;
            resolved
                .matches_usage_target(target)
                .then_some(CargoTargetLocation {
                    cargo_toml_path: canonicalize_or_original(cargo_toml_path.to_path_buf()),
                    range: feature_string.unquoted_range(),
                })
        })
        .collect()
}

fn collect_dependency_feature_usage_locations(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    target: &CargoFeatureUsageTarget,
    toml_version: TomlVersion,
) -> Vec<CargoTargetLocation> {
    let mut locations = Vec::new();

    for (dependency_accessors, dependency_value) in dependency_entries(document_tree) {
        let Value::Table(table) = dependency_value else {
            continue;
        };
        let Some(Value::Array(features)) = table.get("features") else {
            continue;
        };

        for feature_value in features.values() {
            let Value::String(feature_string) = feature_value else {
                continue;
            };
            let Some(resolved) = resolve_dependency_feature_string_target(
                document_tree,
                cargo_toml_path,
                dependency_accessors.as_slice(),
                feature_string,
                toml_version,
            ) else {
                continue;
            };
            if resolved.matches_usage_target(target) {
                locations.push(CargoTargetLocation {
                    cargo_toml_path: canonicalize_or_original(cargo_toml_path.to_path_buf()),
                    range: feature_string.unquoted_range(),
                });
            }
        }
    }

    locations
}

fn resolve_feature_table_string_target(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    feature_string: &tombi_document_tree::String,
    toml_version: TomlVersion,
) -> Option<ResolvedCargoFeatureTarget> {
    match parse_cargo_feature_ref(feature_string.value()) {
        CargoFeatureRef::LocalFeature(feature_name) => {
            find_local_feature(document_tree, cargo_toml_path, feature_name)
                .map(|location| ResolvedCargoFeatureTarget::LocalFeature {
                    location,
                    feature_name: feature_name.to_string(),
                })
                .or_else(|| {
                    (!has_explicit_dep_feature(document_tree, feature_name))
                        .then(|| {
                            find_optional_dependency(document_tree, cargo_toml_path, feature_name)
                        })
                        .flatten()
                        .map(|location| ResolvedCargoFeatureTarget::OptionalDependency {
                            location,
                            dep_key: feature_name.to_string(),
                        })
                })
        }
        CargoFeatureRef::OptionalDependency(dep_key) => {
            find_optional_dependency(document_tree, cargo_toml_path, dep_key).map(|location| {
                ResolvedCargoFeatureTarget::OptionalDependency {
                    location,
                    dep_key: dep_key.to_string(),
                }
            })
        }
        CargoFeatureRef::DependencyFeature {
            dep_key, feature, ..
        } => resolve_named_dependency(
            document_tree,
            cargo_toml_path,
            find_non_workspace_dependency_entry(document_tree, dep_key)?.as_slice(),
            toml_version,
        )
        .and_then(|resolved| {
            find_local_feature(&resolved.document_tree, &resolved.cargo_toml_path, feature)
        })
        .map(|location| ResolvedCargoFeatureTarget::LocalFeature {
            location,
            feature_name: feature.to_string(),
        }),
    }
}

fn resolve_dependency_feature_string_target(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &Path,
    dependency_accessors: &[Accessor],
    feature_string: &tombi_document_tree::String,
    toml_version: TomlVersion,
) -> Option<ResolvedCargoFeatureTarget> {
    resolve_named_dependency(
        document_tree,
        cargo_toml_path,
        dependency_accessors,
        toml_version,
    )
    .and_then(|resolved| {
        find_local_feature(
            &resolved.document_tree,
            &resolved.cargo_toml_path,
            feature_string.value(),
        )
    })
    .map(|location| ResolvedCargoFeatureTarget::LocalFeature {
        location,
        feature_name: feature_string.value().to_string(),
    })
}

fn dependency_entries<'a>(
    document_tree: &'a tombi_document_tree::DocumentTree,
) -> Vec<(Vec<Accessor>, &'a tombi_document_tree::Value)> {
    let mut entries = Vec::new();

    if let Some((_, Value::Table(workspace_dependencies))) =
        dig_keys(document_tree, &["workspace", "dependencies"])
    {
        for (dependency_key, dependency_value) in workspace_dependencies.key_values() {
            entries.push((
                vec![
                    Accessor::Key("workspace".to_string()),
                    Accessor::Key("dependencies".to_string()),
                    Accessor::Key(dependency_key.value.to_string()),
                ],
                dependency_value,
            ));
        }
    }

    for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some((_, Value::Table(dependencies))) = dig_keys(document_tree, &[dependency_kind]) {
            for (dependency_key, dependency_value) in dependencies.key_values() {
                entries.push((
                    vec![
                        Accessor::Key(dependency_kind.to_string()),
                        Accessor::Key(dependency_key.value.to_string()),
                    ],
                    dependency_value,
                ));
            }
        }
    }

    if let Some((_, Value::Table(targets))) = dig_keys(document_tree, &["target"]) {
        for (target_key, target_value) in targets.key_values() {
            let Value::Table(target_table) = target_value else {
                continue;
            };
            for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
                let Some(Value::Table(dependencies)) = target_table.get(dependency_kind) else {
                    continue;
                };
                for (dependency_key, dependency_value) in dependencies.key_values() {
                    entries.push((
                        vec![
                            Accessor::Key("target".to_string()),
                            Accessor::Key(target_key.value.to_string()),
                            Accessor::Key(dependency_kind.to_string()),
                            Accessor::Key(dependency_key.value.to_string()),
                        ],
                        dependency_value,
                    ));
                }
            }
        }
    }

    entries
}

fn workspace_manifest_paths(
    current_document_tree: &tombi_document_tree::DocumentTree,
    current_cargo_toml_path: &Path,
    toml_version: TomlVersion,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let current_cargo_toml_path = canonicalize_or_original(current_cargo_toml_path.to_path_buf());

    if current_document_tree.contains_key("workspace") {
        collect_workspace_paths(current_document_tree, &current_cargo_toml_path, &mut paths);
    } else if let Some((workspace_cargo_toml_path, _, workspace_document_tree)) =
        find_workspace_cargo_toml(
            &current_cargo_toml_path,
            get_workspace_cargo_toml_path(current_document_tree),
            toml_version,
        )
    {
        collect_workspace_paths(
            &workspace_document_tree,
            &workspace_cargo_toml_path,
            &mut paths,
        );
    }

    if !paths.contains(&current_cargo_toml_path) {
        paths.push(current_cargo_toml_path);
    }

    paths
}

fn collect_workspace_paths(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace_cargo_toml_path: &Path,
    paths: &mut Vec<PathBuf>,
) {
    let workspace_cargo_toml_path =
        canonicalize_or_original(workspace_cargo_toml_path.to_path_buf());
    if !paths.contains(&workspace_cargo_toml_path) {
        paths.push(workspace_cargo_toml_path.clone());
    }

    let Some(workspace_dir_path) = workspace_cargo_toml_path.parent() else {
        return;
    };

    let member_accessors = [
        Accessor::Key("workspace".to_string()),
        Accessor::Key("members".to_string()),
    ];
    let member_patterns =
        extract_member_patterns(workspace_document_tree, &member_accessors, "members");
    let exclude_patterns = extract_exclude_patterns(workspace_document_tree);

    for (_, member_cargo_toml_path) in
        find_package_cargo_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
    {
        let member_cargo_toml_path = canonicalize_or_original(member_cargo_toml_path);
        if !paths.contains(&member_cargo_toml_path) {
            paths.push(member_cargo_toml_path);
        }
    }
}
