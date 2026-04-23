use std::{borrow::Cow, str::FromStr};

use crate::{
    feature_navigation::{CargoFeatureRef, parse_cargo_feature_ref},
    find_cargo_toml, find_package_cargo_toml_paths, find_workspace_cargo_toml,
    get_uri_relative_to_cargo_toml, get_workspace_cargo_toml_path, load_cargo_toml,
    resolve_dependency_feature_string, resolve_feature_table_string,
};
use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;
use tombi_schema_store::Accessor;

type RegistryMap = tombi_hashmap::HashMap<String, Registry>;
const DEFAULT_REGISTRY_INDEX: &str = "https://crates.io/crates";

struct Registry {
    index: String,
}

#[derive(Clone, Copy)]
pub enum DocumentLinkToolTip {
    GitRepository,
    Registry,
    CrateIo,
    CargoToml,
    CargoTomlFirstMember,
    WorkspaceCargoToml,
    PathFile,
}

impl From<&DocumentLinkToolTip> for &'static str {
    #[inline]
    fn from(val: &DocumentLinkToolTip) -> Self {
        match val {
            DocumentLinkToolTip::GitRepository => "Open Git Repository",
            DocumentLinkToolTip::Registry => "Open Registry",
            DocumentLinkToolTip::CrateIo => "Open crate.io",
            DocumentLinkToolTip::CargoToml => "Open Cargo.toml",
            DocumentLinkToolTip::CargoTomlFirstMember => "Open first Cargo.toml in members",
            DocumentLinkToolTip::WorkspaceCargoToml => "Open Workspace Cargo.toml",
            DocumentLinkToolTip::PathFile => "Open Path File",
        }
    }
}

impl From<DocumentLinkToolTip> for &'static str {
    #[inline]
    fn from(val: DocumentLinkToolTip) -> Self {
        (&val).into()
    }
}

impl From<DocumentLinkToolTip> for Cow<'static, str> {
    #[inline]
    fn from(val: DocumentLinkToolTip) -> Self {
        Cow::Borrowed(val.into())
    }
}

impl std::fmt::Display for DocumentLinkToolTip {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(self))
    }
}

pub async fn document_link(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Option<Vec<tombi_extension::DocumentLink>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is Cargo.toml
    if !text_document_uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }
    let Some(cargo_toml_path) = text_document_uri.to_file_path().ok() else {
        return Ok(None);
    };

    if !features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.document_link())
        .map(|document_link| document_link.enabled())
        .unwrap_or_default()
        .value()
    {
        return Ok(None);
    }

    let mut document_links = vec![];

    if document_tree.contains_key("workspace") {
        document_links.extend(document_link_for_workspace_cargo_toml(
            document_tree,
            &cargo_toml_path,
            toml_version,
            features,
        )?);

        // For Root Package
        // See: https://doc.rust-lang.org/cargo/reference/workspaces.html#root-package
        if document_tree.contains_key("package") {
            document_links.extend(document_link_for_crate_cargo_toml(
                document_tree,
                &cargo_toml_path,
                toml_version,
                features,
            )?);
        }
    } else {
        document_links.extend(document_link_for_crate_cargo_toml(
            document_tree,
            &cargo_toml_path,
            toml_version,
            features,
        )?);
    }

    if document_links.is_empty() {
        return Ok(None);
    }

    Ok(Some(document_links))
}

fn document_link_for_workspace_cargo_toml(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut total_document_links = vec![];

    let registries = get_registries(workspace_cargo_toml_path, toml_version).unwrap_or_default();

    if (cargo_toml_document_link_enabled(features)
        || git_document_link_enabled(features)
        || path_document_link_enabled(features)
        || crates_io_document_link_enabled(features))
        && let Some((_, tombi_document_tree::Value::Table(dependencies))) =
            dig_keys(workspace_document_tree, &["workspace", "dependencies"])
    {
        total_document_links.extend(document_link_for_workspace_depencencies(
            dependencies,
            workspace_cargo_toml_path,
            &registries,
            toml_version,
            features,
        )?);
    }
    if cargo_toml_document_link_enabled(features) {
        total_document_links.extend(create_member_document_links(
            workspace_document_tree,
            "members",
            workspace_cargo_toml_path,
            toml_version,
        ));
        total_document_links.extend(create_member_document_links(
            workspace_document_tree,
            "default-members",
            workspace_cargo_toml_path,
            toml_version,
        ));
        total_document_links.extend(document_link_for_workspace_dependency_features(
            workspace_document_tree,
            workspace_cargo_toml_path,
            toml_version,
            features,
        ));
    }

    Ok(total_document_links)
}

fn create_member_document_links(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    members_key: &str,
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Vec<tombi_extension::DocumentLink> {
    let Some((_, tombi_document_tree::Value::Array(members))) =
        dig_keys(workspace_document_tree, &["workspace", members_key])
    else {
        return Vec::with_capacity(0);
    };

    let mut document_links = Vec::new();
    let exclude_patterns: Vec<_> =
        match dig_keys(workspace_document_tree, &["workspace", "exclude"]) {
            Some((_, tombi_document_tree::Value::Array(exclude))) => exclude
                .iter()
                .filter_map(|member| match member {
                    tombi_document_tree::Value::String(member_pattern) => Some(member_pattern),
                    _ => None,
                })
                .collect(),
            _ => Vec::with_capacity(0),
        };

    for member in members.iter() {
        let member = match member {
            tombi_document_tree::Value::String(member) => member,
            _ => continue,
        };

        let member_patterns = vec![member];

        let Some(workspace_dir_path) = workspace_cargo_toml_path.parent() else {
            continue;
        };

        let mut member_document_links: Vec<_> =
            find_package_cargo_toml_paths(&member_patterns, &exclude_patterns, workspace_dir_path)
                .filter_map(|(_, cargo_toml_path)| {
                    let (_, cargo_toml_document_tree) =
                        load_cargo_toml(&cargo_toml_path, toml_version)?;
                    let (_, package_name) =
                        dig_keys(&cargo_toml_document_tree, &["package", "name"])?;
                    let package_name = match package_name {
                        tombi_document_tree::Value::String(s) => s,
                        _ => return None,
                    };

                    let mut target = tombi_uri::Uri::from_file_path(&cargo_toml_path).ok()?;
                    target.set_fragment(Some(&format!(
                        "L{}",
                        package_name.unquoted_range().start.line + 1
                    )));

                    Some(tombi_extension::DocumentLink {
                        target,
                        range: member.unquoted_range(),
                        tooltip: DocumentLinkToolTip::CargoTomlFirstMember.into(),
                    })
                })
                .collect();

        match member_document_links.len() {
            0 => {}
            1 => {
                if let Some(ref mut document_link) = member_document_links.first_mut() {
                    document_link.tooltip = DocumentLinkToolTip::CargoToml.into();
                }
                document_links.extend(member_document_links);
            }
            _ => {
                // only one link is given
                if let Some(document_link) = member_document_links.into_iter().next() {
                    document_links.push(document_link);
                }
            }
        }
    }

    document_links
}

fn document_link_for_workspace_depencencies(
    dependencies: &tombi_document_tree::Table,
    workspace_cargo_toml_path: &std::path::Path,
    registries: &RegistryMap,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut total_document_links = vec![];
    for (crate_name, crate_value) in dependencies.key_values() {
        if let Ok(document_links) = document_link_for_workspace_dependency(
            crate_name,
            crate_value,
            workspace_cargo_toml_path,
            registries,
            toml_version,
            features,
        ) {
            total_document_links.extend(document_links);
        }
    }

    Ok(total_document_links)
}

fn document_link_for_crate_cargo_toml(
    crate_document_tree: &tombi_document_tree::DocumentTree,
    crate_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut total_dependencies = vec![];
    for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some((_, tombi_document_tree::Value::Table(dependencies))) =
            dig_keys(crate_document_tree, &[key])
        {
            total_dependencies.extend(dependencies.key_values());
        }
        if let Some((_, tombi_document_tree::Value::Table(targets))) =
            dig_keys(crate_document_tree, &["target"])
        {
            for value in targets.values() {
                let tombi_document_tree::Value::Table(platform) = value else {
                    continue;
                };
                if let Some((_, tombi_document_tree::Value::Table(dependencies))) =
                    dig_keys(platform, &[key])
                {
                    total_dependencies.extend(dependencies.key_values());
                }
            }
        }
    }

    let mut total_document_links = vec![];
    if let Some((workspace_cargo_toml_path, _, workspace_document_tree)) = find_workspace_cargo_toml(
        crate_cargo_toml_path,
        get_workspace_cargo_toml_path(crate_document_tree),
        toml_version,
    ) {
        let registries =
            get_registries(&workspace_cargo_toml_path, toml_version).unwrap_or_default();

        // Support Workspace
        // See: https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field
        if cargo_toml_document_link_enabled(features)
            && let Some((_, tombi_document_tree::Value::String(workspace_path))) =
                dig_keys(crate_document_tree, &["package", "workspace"])
            && let Ok(target) = tombi_uri::Uri::from_file_path(&workspace_cargo_toml_path)
        {
            total_document_links.push(tombi_extension::DocumentLink {
                target,
                range: workspace_path.unquoted_range(),
                tooltip: DocumentLinkToolTip::WorkspaceCargoToml.into(),
            });
        }

        // Support Package Table
        // See: https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table
        if workspace_document_link_enabled(features) {
            for package_item in [
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
            ] {
                if let (
                    Some((workspace_key, tombi_document_tree::Value::Boolean(value))),
                    Some((package_item_key, _)),
                ) = (
                    dig_keys(crate_document_tree, &["package", package_item, "workspace"]),
                    dig_keys(
                        &workspace_document_tree,
                        &["workspace", "package", package_item],
                    ),
                ) {
                    let Ok(mut target) = tombi_uri::Uri::from_file_path(&workspace_cargo_toml_path)
                    else {
                        continue;
                    };
                    target.set_fragment(Some(&format!(
                        "L{}",
                        package_item_key.range().start.line + 1
                    )));
                    total_document_links.push(tombi_extension::DocumentLink {
                        target,
                        range: workspace_key.range() + value.range(),
                        tooltip: DocumentLinkToolTip::WorkspaceCargoToml.into(),
                    });
                }
            }
        }

        // Support Lints Workspace
        // See: https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table
        if workspace_document_link_enabled(features)
            && let (
                Some((workspace_key, tombi_document_tree::Value::Boolean(value))),
                Some((workspace_lints_key, _)),
            ) = (
                dig_keys(crate_document_tree, &["lints", "workspace"]),
                dig_keys(&workspace_document_tree, &["workspace", "lints"]),
            )
            && let Ok(mut target) = tombi_uri::Uri::from_file_path(&workspace_cargo_toml_path)
        {
            target.set_fragment(Some(&format!(
                "L{}",
                workspace_lints_key.range().start.line + 1
            )));
            total_document_links.push(tombi_extension::DocumentLink {
                target,
                range: workspace_key.range() + value.range(),
                tooltip: DocumentLinkToolTip::WorkspaceCargoToml.into(),
            });
        };

        // Support Workspace Dependencies
        let workspace_dependencies =
            if let Some((_, tombi_document_tree::Value::Table(dependencies))) =
                dig_keys(&workspace_document_tree, &["workspace", "dependencies"])
            {
                Some(dependencies)
            } else {
                None
            };
        for (crate_key, crate_value) in total_dependencies {
            if let Ok(document_links) = document_link_for_crate_dependency_has_workspace(
                crate_key,
                crate_value,
                crate_cargo_toml_path,
                workspace_dependencies,
                &workspace_cargo_toml_path,
                &registries,
                toml_version,
                features,
            ) {
                total_document_links.extend(document_links);
            }
        }
    } else {
        let registries = get_registries(crate_cargo_toml_path, toml_version).unwrap_or_default();

        for (crate_key, crate_value) in total_dependencies {
            if let Ok(document_links) = document_link_for_dependency(
                crate_key,
                crate_value,
                crate_cargo_toml_path,
                &registries,
                toml_version,
                features,
            ) {
                total_document_links.extend(document_links);
            }
        }
    }

    if path_document_link_enabled(features) {
        total_document_links.extend(document_link_for_bin_targets(
            crate_document_tree,
            crate_cargo_toml_path,
        ));
    }
    if cargo_toml_document_link_enabled(features) {
        total_document_links.extend(document_link_for_feature_table_strings(
            crate_document_tree,
            crate_cargo_toml_path,
            toml_version,
            features,
        ));
        total_document_links.extend(document_link_for_crate_dependency_features(
            crate_document_tree,
            crate_cargo_toml_path,
            toml_version,
            features,
        ));
    }

    Ok(total_document_links)
}

fn document_link_for_feature_table_strings(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Vec<tombi_extension::DocumentLink> {
    let Some((_, tombi_document_tree::Value::Table(features_table))) =
        dig_keys(document_tree, &["features"])
    else {
        return Vec::new();
    };

    features_table
        .values()
        .filter_map(|value| match value {
            tombi_document_tree::Value::Array(features) => Some(features),
            _ => None,
        })
        .flat_map(|features| features.values())
        .filter_map(|value| match value {
            tombi_document_tree::Value::String(feature_string) => Some(feature_string),
            _ => None,
        })
        .filter_map(|feature_string| {
            if matches!(
                parse_cargo_feature_ref(feature_string.value()),
                CargoFeatureRef::OptionalDependency(_)
            ) {
                return None;
            }
            let target = resolve_feature_table_string(
                document_tree,
                cargo_toml_path,
                feature_string,
                toml_version,
            )?;
            if !cargo_toml_document_link_enabled(features) {
                return None;
            }
            cargo_toml_document_link(
                feature_string.unquoted_range(),
                &target,
                DocumentLinkToolTip::CargoToml,
            )
        })
        .collect()
}

fn document_link_for_workspace_dependency_features(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Vec<tombi_extension::DocumentLink> {
    let Some((_, tombi_document_tree::Value::Table(dependencies))) =
        dig_keys(document_tree, &["workspace", "dependencies"])
    else {
        return Vec::new();
    };

    document_link_for_dependency_table_features(
        document_tree,
        cargo_toml_path,
        toml_version,
        features,
        dependencies,
        |dependency_key| {
            vec![
                Accessor::Key("workspace".to_string()),
                Accessor::Key("dependencies".to_string()),
                Accessor::Key(dependency_key.to_string()),
            ]
        },
    )
}

fn document_link_for_crate_dependency_features(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Vec<tombi_extension::DocumentLink> {
    let mut document_links = Vec::new();

    for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some((_, tombi_document_tree::Value::Table(dependencies))) =
            dig_keys(document_tree, &[dependency_kind])
        {
            document_links.extend(document_link_for_dependency_table_features(
                document_tree,
                cargo_toml_path,
                toml_version,
                features,
                dependencies,
                |dependency_key| {
                    vec![
                        Accessor::Key(dependency_kind.to_string()),
                        Accessor::Key(dependency_key.to_string()),
                    ]
                },
            ));
        }
    }

    if let Some((_, tombi_document_tree::Value::Table(targets))) =
        dig_keys(document_tree, &["target"])
    {
        for (target_key, target_value) in targets.key_values() {
            let tombi_document_tree::Value::Table(target_table) = target_value else {
                continue;
            };
            for dependency_kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
                let Some(tombi_document_tree::Value::Table(dependencies)) =
                    target_table.get(dependency_kind)
                else {
                    continue;
                };
                document_links.extend(document_link_for_dependency_table_features(
                    document_tree,
                    cargo_toml_path,
                    toml_version,
                    features,
                    dependencies,
                    |dependency_key| {
                        vec![
                            Accessor::Key("target".to_string()),
                            Accessor::Key(target_key.value.to_string()),
                            Accessor::Key(dependency_kind.to_string()),
                            Accessor::Key(dependency_key.to_string()),
                        ]
                    },
                ));
            }
        }
    }

    document_links
}

fn document_link_for_dependency_table_features<F>(
    document_tree: &tombi_document_tree::DocumentTree,
    cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
    extension_features: Option<&tombi_config::CargoExtensionFeatures>,
    dependencies: &tombi_document_tree::Table,
    dependency_accessors: F,
) -> Vec<tombi_extension::DocumentLink>
where
    F: Fn(&str) -> Vec<Accessor>,
{
    dependencies
        .key_values()
        .iter()
        .flat_map(|(dependency_key, dependency_value)| {
            let tombi_document_tree::Value::Table(table) = dependency_value else {
                return Vec::new();
            };
            let Some(tombi_document_tree::Value::Array(features)) = table.get("features") else {
                return Vec::new();
            };
            features
                .values()
                .iter()
                .filter_map(|feature_value| match feature_value {
                    tombi_document_tree::Value::String(feature_string) => {
                        let target = resolve_dependency_feature_string(
                            document_tree,
                            cargo_toml_path,
                            dependency_accessors(dependency_key.value.as_str()).as_slice(),
                            feature_string,
                            toml_version,
                        )?;
                        if !cargo_toml_document_link_enabled(extension_features) {
                            return None;
                        }
                        cargo_toml_document_link(
                            feature_string.unquoted_range(),
                            &target,
                            DocumentLinkToolTip::CargoToml,
                        )
                    }
                    _ => None,
                })
                .collect_vec()
        })
        .collect()
}

fn cargo_toml_document_link(
    range: tombi_text::Range,
    target: &crate::CargoTargetLocation,
    tooltip: DocumentLinkToolTip,
) -> Option<tombi_extension::DocumentLink> {
    let mut target_uri = tombi_uri::Uri::from_file_path(&target.cargo_toml_path).ok()?;
    target_uri.set_fragment(Some(&format!("L{}", target.range.start.line + 1)));
    Some(tombi_extension::DocumentLink {
        target: target_uri,
        range,
        tooltip: tooltip.into(),
    })
}

fn dependency_table_tooltip(table: &tombi_document_tree::Table) -> DocumentLinkToolTip {
    if table.contains_key("path") {
        DocumentLinkToolTip::PathFile
    } else {
        DocumentLinkToolTip::CargoToml
    }
}

fn dependency_uses_local_path(crate_value: &tombi_document_tree::Value) -> bool {
    matches!(
        crate_value,
        tombi_document_tree::Value::Table(table) if table.contains_key("path")
    )
}

fn workspace_dependency_target(
    crate_key: &tombi_document_tree::Key,
    crate_value: &tombi_document_tree::Value,
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Option<crate::CargoTargetLocation> {
    let tombi_document_tree::Value::Table(table) = crate_value else {
        return None;
    };
    let tombi_document_tree::Value::String(crate_path) = table.get("path")? else {
        return None;
    };
    let (cargo_toml_path, _, document_tree) = find_cargo_toml(
        workspace_cargo_toml_path,
        std::path::Path::new(crate_path.value()),
        toml_version,
    )?;
    let Some((package_name_key, tombi_document_tree::Value::String(package_name))) =
        dig_keys(&document_tree, &["package", "name"])
    else {
        return None;
    };
    let package_name_matches =
        if let Some(tombi_document_tree::Value::String(real_package_name)) = table.get("package") {
            package_name.value() == real_package_name.value()
        } else {
            package_name.value() == crate_key.value
        };

    package_name_matches.then(|| crate::CargoTargetLocation {
        cargo_toml_path,
        range: package_name_key.range(),
    })
}

fn dependency_key_tooltip(
    tooltip: std::borrow::Cow<'static, str>,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Option<std::borrow::Cow<'static, str>> {
    if tooltip_matches(&tooltip, DocumentLinkToolTip::PathFile) {
        cargo_toml_document_link_enabled(features)
            .then(|| std::borrow::Cow::Borrowed(DocumentLinkToolTip::CargoToml.into()))
    } else {
        Some(tooltip)
    }
}

fn tooltip_matches(tooltip: &str, expected: DocumentLinkToolTip) -> bool {
    tooltip == Into::<&'static str>::into(expected)
}

fn document_link_for_workspace_dependency(
    crate_key: &tombi_document_tree::Key,
    crate_value: &tombi_document_tree::Value,
    workspace_cargo_toml_path: &std::path::Path,
    registries: &RegistryMap,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let document_links = document_link_for_dependency(
        crate_key,
        crate_value,
        workspace_cargo_toml_path,
        registries,
        toml_version,
        features,
    )?;

    if document_links.is_empty() {
        if dependency_uses_local_path(crate_value) {
            return Ok(Vec::new());
        }
        Ok((crates_io_document_link_enabled(features))
            .then(|| get_crate_io_crate_link(crate_key, crate_value))
            .flatten()
            .into_iter()
            .collect_vec())
    } else {
        Ok(document_links
            .into_iter()
            .flat_map(|document_link| {
                let mut links = Vec::with_capacity(2);
                if let Some(tooltip) =
                    dependency_key_tooltip(document_link.tooltip.clone(), features)
                {
                    links.push(tombi_extension::DocumentLink {
                        target: document_link.target.clone(),
                        range: crate_key.unquoted_range(),
                        tooltip,
                    });
                }
                links.push(document_link);
                links
            })
            .collect())
    }
}

fn document_link_for_crate_dependency_has_workspace(
    crate_key: &tombi_document_tree::Key,
    crate_value: &tombi_document_tree::Value,
    crate_cargo_toml_path: &std::path::Path,
    workspace_dependencies: Option<&tombi_document_tree::Table>,
    workspace_cargo_toml_path: &std::path::Path,
    registries: &RegistryMap,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let document_links = document_link_for_dependency(
        crate_key,
        crate_value,
        crate_cargo_toml_path,
        registries,
        toml_version,
        features,
    )?;

    if !document_links.is_empty() {
        return Ok(document_links
            .into_iter()
            .flat_map(|document_link| {
                let mut links = Vec::with_capacity(2);
                if let Some(tooltip) =
                    dependency_key_tooltip(document_link.tooltip.clone(), features)
                {
                    links.push(tombi_extension::DocumentLink {
                        target: document_link.target.clone(),
                        range: crate_key.unquoted_range(),
                        tooltip,
                    });
                }
                links.push(document_link);
                links
            })
            .collect());
    }

    if let (tombi_document_tree::Value::Table(table), Some(workspace_dependencies)) =
        (crate_value, workspace_dependencies)
        && let Some((workspace_key, tombi_document_tree::Value::Boolean(is_workspace))) =
            table.get_key_value("workspace")
        && is_workspace.value()
        && let Some(workspace_crate_value) = workspace_dependencies.get(&crate_key)
    {
        let mut document_links = if dependency_uses_local_path(workspace_crate_value) {
            cargo_toml_document_link_enabled(features)
                .then(|| {
                    workspace_dependency_target(
                        crate_key,
                        workspace_crate_value,
                        workspace_cargo_toml_path,
                        toml_version,
                    )
                })
                .flatten()
                .and_then(|target_location| {
                    cargo_toml_document_link(
                        crate_key.unquoted_range(),
                        &target_location,
                        DocumentLinkToolTip::CargoToml,
                    )
                })
                .into_iter()
                .collect_vec()
        } else {
            document_link_for_workspace_dependency(
                crate_key,
                workspace_crate_value,
                workspace_cargo_toml_path,
                registries,
                toml_version,
                features,
            )?
            .into_iter()
            .next()
            .into_iter()
            .collect_vec()
        };
        let tooltip = dependency_table_tooltip(table);
        for document_link in &mut document_links {
            if tooltip_matches(&document_link.tooltip, DocumentLinkToolTip::CargoToml)
                || tooltip_matches(&document_link.tooltip, DocumentLinkToolTip::PathFile)
            {
                document_link.tooltip = std::borrow::Cow::Borrowed((&tooltip).into());
            }
        }

        if workspace_document_link_enabled(features)
            && let Ok(mut target) = tombi_uri::Uri::from_file_path(workspace_cargo_toml_path)
        {
            target.set_fragment(Some(&format!(
                "L{}",
                workspace_crate_value.range().start.line + 1
            )));
            document_links.push(tombi_extension::DocumentLink {
                target,
                range: workspace_key.range() + is_workspace.range(),
                tooltip: DocumentLinkToolTip::WorkspaceCargoToml.into(),
            });
        }

        return Ok(document_links);
    }

    Ok((crates_io_document_link_enabled(features))
        .then(|| get_crate_io_crate_link(crate_key, crate_value))
        .flatten()
        .into_iter()
        .collect_vec())
}

fn document_link_for_bin_targets(
    crate_document_tree: &tombi_document_tree::DocumentTree,
    crate_cargo_toml_path: &std::path::Path,
) -> Vec<tombi_extension::DocumentLink> {
    let Some((_, tombi_document_tree::Value::Array(bin_items))) =
        dig_keys(crate_document_tree, &["bin"])
    else {
        return Vec::with_capacity(0);
    };

    bin_items
        .values()
        .iter()
        .filter_map(|value| match value {
            tombi_document_tree::Value::Table(bin_table) => bin_table.get_key_value("path"),
            _ => None,
        })
        .filter_map(|(_, path_value)| match path_value {
            tombi_document_tree::Value::String(path_string) => Some(path_string),
            _ => None,
        })
        .filter_map(|path_string| {
            let raw_path = path_string.value();
            let target = get_uri_relative_to_cargo_toml(
                std::path::Path::new(raw_path),
                crate_cargo_toml_path,
            )?;

            Some(tombi_extension::DocumentLink {
                target,
                range: path_string.unquoted_range(),
                tooltip: DocumentLinkToolTip::PathFile.into(),
            })
        })
        .collect()
}

fn document_link_for_dependency(
    crate_key: &tombi_document_tree::Key,
    crate_value: &tombi_document_tree::Value,
    crate_cargo_toml_path: &std::path::Path,
    registries: &RegistryMap,
    toml_version: TomlVersion,
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut document_links = Vec::new();
    let mut package_name = crate_key.value.as_str();
    if let tombi_document_tree::Value::Table(table) = crate_value {
        if let Some(tombi_document_tree::Value::String(real_package)) = table.get("package") {
            package_name = real_package.value();
        };

        if path_document_link_enabled(features)
            && let Some(tombi_document_tree::Value::String(crate_path)) = table.get("path")
            && let Some((path_target_cargo_toml_path, _, path_target_document_tree)) =
                find_cargo_toml(
                    crate_cargo_toml_path,
                    std::path::Path::new(crate_path.value()),
                    toml_version,
                )
            && let Some((package_name_key, tombi_document_tree::Value::String(package_name))) =
                tombi_document_tree::dig_keys(&path_target_document_tree, &["package", "name"])
        {
            let package_name_check =
                if let Some(tombi_document_tree::Value::String(real_package_name)) =
                    table.get("package")
                {
                    real_package_name.value() == crate_key.value
                } else {
                    package_name.value() == crate_key.value
                };
            if package_name_check {
                let Ok(mut target) = tombi_uri::Uri::from_file_path(path_target_cargo_toml_path)
                else {
                    return Ok(Vec::new());
                };
                target.set_fragment(Some(&format!(
                    "L{}",
                    package_name_key.range().start.line + 1
                )));

                document_links.push(tombi_extension::DocumentLink {
                    target,
                    range: crate_path.unquoted_range(),
                    tooltip: DocumentLinkToolTip::PathFile.into(),
                });
            }
        }

        if git_document_link_enabled(features)
            && let Some(tombi_document_tree::Value::String(git_url)) = table.get("git")
        {
            let target = if let Ok(target) = tombi_uri::Uri::from_str(git_url.value()) {
                target
            } else if let Ok(target) = tombi_uri::Uri::from_file_path(git_url.value()) {
                target
            } else {
                return Ok(document_links);
            };

            document_links.push(tombi_extension::DocumentLink {
                target,
                range: git_url.unquoted_range(),
                tooltip: DocumentLinkToolTip::GitRepository.into(),
            });
        }

        if crates_io_document_link_enabled(features)
            && let Some(tombi_document_tree::Value::String(registry_name)) = table.get("registry")
            && let Some(registry) = registries.get(registry_name.value())
            && let Ok(target) =
                tombi_uri::Uri::from_str(&format!("{}/{}", registry.index, package_name))
        {
            document_links.push(tombi_extension::DocumentLink {
                target,
                range: registry_name.unquoted_range(),
                tooltip: DocumentLinkToolTip::CrateIo.into(),
            });
        }
    }

    Ok(document_links)
}

fn get_registries(
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<RegistryMap, tower_lsp::jsonrpc::Error> {
    let mut registries = RegistryMap::default();
    if let Some((_, cargo_toml_document_tree)) = load_cargo_toml(
        &workspace_cargo_toml_path.join(".cargo/config.toml"),
        toml_version,
    ) && let Some(tombi_document_tree::Value::Table(registories_table)) =
        cargo_toml_document_tree.get("registries")
    {
        for (name, value) in registories_table.key_values() {
            if let tombi_document_tree::Value::Table(table) = value
                && let Some(tombi_document_tree::Value::String(index)) = table.get("index")
            {
                registries.insert(
                    name.value.to_owned(),
                    Registry {
                        index: index.value().into(),
                    },
                );
            }
        }
    }

    Ok(registries)
}

#[inline]
fn cargo_toml_document_link_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.document_link())
        .and_then(|document_link| document_link.cargo_toml())
        .map(|cargo_toml| cargo_toml.enabled())
        .unwrap_or_default()
        .value()
}

#[inline]
fn workspace_document_link_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.document_link())
        .and_then(|document_link| document_link.workspace())
        .map(|workspace| workspace.enabled())
        .unwrap_or_default()
        .value()
}

#[inline]
fn git_document_link_enabled(features: Option<&tombi_config::CargoExtensionFeatures>) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.document_link())
        .and_then(|document_link| document_link.git())
        .map(|git| git.enabled())
        .unwrap_or_default()
        .value()
}

#[inline]
fn path_document_link_enabled(features: Option<&tombi_config::CargoExtensionFeatures>) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.document_link())
        .and_then(|document_link| document_link.path())
        .map(|path| path.enabled())
        .unwrap_or_default()
        .value()
}

#[inline]
fn crates_io_document_link_enabled(
    features: Option<&tombi_config::CargoExtensionFeatures>,
) -> bool {
    features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.document_link())
        .and_then(|document_link| document_link.crates_io())
        .map(|crates_io| crates_io.enabled())
        .unwrap_or_default()
        .value()
}

fn get_crate_io_crate_link(
    crate_key: &tombi_document_tree::Key,
    crate_value: &tombi_document_tree::Value,
) -> Option<tombi_extension::DocumentLink> {
    let mut crate_name = crate_key.value.as_str();
    if let tombi_document_tree::Value::Table(table) = crate_value
        && let Some(tombi_document_tree::Value::String(real_package)) = table.get("package")
    {
        crate_name = real_package.value();
    }

    tombi_uri::Uri::from_str(&format!("{DEFAULT_REGISTRY_INDEX}/{crate_name}"))
        .map(|target| tombi_extension::DocumentLink {
            target,
            range: crate_key.unquoted_range(),
            tooltip: DocumentLinkToolTip::CrateIo.into(),
        })
        .ok()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tombi_config::TomlVersion;
    use tombi_document_tree::dig_keys;

    use crate::document_link::{
        cargo_toml_document_link_enabled, crates_io_document_link_enabled, dependency_key_tooltip,
        git_document_link_enabled, path_document_link_enabled, workspace_document_link_enabled,
    };

    use super::{RegistryMap, document_link_for_dependency};

    fn disabled_path_link_features() -> tombi_config::CargoExtensionFeatures {
        tombi_config::CargoExtensionFeatures::Features(tombi_config::CargoExtensionFeatureTree {
            lsp: Some(tombi_config::CargoLspFeatures::Features(
                tombi_config::CargoLspFeatureTree {
                    completion: None,
                    inlay_hint: None,
                    goto_definition: None,
                    goto_declaration: None,
                    document_link: Some(tombi_config::CargoDocumentLinkFeatures::Features(
                        tombi_config::CargoDocumentLinkFeatureTree {
                            cargo_toml: Some(tombi_config::ToggleFeatureDefaultFalse {
                                enabled: Some(true.into()),
                            }),
                            workspace: None,
                            git: None,
                            path: Some(tombi_config::ToggleFeatureDefaultFalse {
                                enabled: Some(false.into()),
                            }),
                            crates_io: None,
                        },
                    )),
                    hover: None,
                    code_action: None,
                },
            )),
        })
    }

    fn disabled_cargo_toml_link_features() -> tombi_config::CargoExtensionFeatures {
        tombi_config::CargoExtensionFeatures::Features(tombi_config::CargoExtensionFeatureTree {
            lsp: Some(tombi_config::CargoLspFeatures::Features(
                tombi_config::CargoLspFeatureTree {
                    completion: None,
                    inlay_hint: None,
                    goto_definition: None,
                    goto_declaration: None,
                    document_link: Some(tombi_config::CargoDocumentLinkFeatures::Features(
                        tombi_config::CargoDocumentLinkFeatureTree {
                            cargo_toml: Some(tombi_config::ToggleFeatureDefaultFalse {
                                enabled: Some(false.into()),
                            }),
                            workspace: Some(tombi_config::ToggleFeatureDefaultFalse {
                                enabled: Some(true.into()),
                            }),
                            git: None,
                            path: Some(tombi_config::ToggleFeatureDefaultFalse {
                                enabled: Some(true.into()),
                            }),
                            crates_io: None,
                        },
                    )),
                    hover: None,
                    code_action: None,
                },
            )),
        })
    }

    fn disabled_workspace_link_features() -> tombi_config::CargoExtensionFeatures {
        tombi_config::CargoExtensionFeatures::Features(tombi_config::CargoExtensionFeatureTree {
            lsp: Some(tombi_config::CargoLspFeatures::Features(
                tombi_config::CargoLspFeatureTree {
                    completion: None,
                    inlay_hint: None,
                    goto_definition: None,
                    goto_declaration: None,
                    document_link: Some(tombi_config::CargoDocumentLinkFeatures::Features(
                        tombi_config::CargoDocumentLinkFeatureTree {
                            cargo_toml: None,
                            workspace: Some(tombi_config::ToggleFeatureDefaultFalse {
                                enabled: Some(false.into()),
                            }),
                            git: None,
                            path: None,
                            crates_io: None,
                        },
                    )),
                    hover: None,
                    code_action: None,
                },
            )),
        })
    }

    #[test]
    fn feature_document_link_allows_same_file_when_path_links_disabled() {
        let features = disabled_path_link_features();

        assert!(cargo_toml_document_link_enabled(Some(&features)));
    }

    #[test]
    fn feature_document_link_allows_cross_file_when_path_links_disabled() {
        let features = disabled_path_link_features();

        assert!(cargo_toml_document_link_enabled(Some(&features),));
    }

    #[test]
    fn workspace_document_link_uses_independent_setting() {
        let features = disabled_cargo_toml_link_features();

        assert!(workspace_document_link_enabled(Some(&features)));

        let features = disabled_workspace_link_features();

        assert!(!workspace_document_link_enabled(Some(&features)));
    }

    #[test]
    fn path_dependency_uses_path_setting_instead_of_cargo_toml_setting() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source_cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let member_dir = temp_dir.path().join("member");
        let member_cargo_toml_path = member_dir.join("Cargo.toml");

        fs::create_dir_all(&member_dir).unwrap();
        fs::write(
            &source_cargo_toml_path,
            r#"
[package]
name = "app"
version = "0.1.0"

[dependencies]
member = { path = "member" }
"#,
        )
        .unwrap();
        fs::write(
            &member_cargo_toml_path,
            r#"
[package]
name = "member"
version = "0.1.0"
"#,
        )
        .unwrap();

        let (_, document_tree) =
            crate::load_cargo_toml(&source_cargo_toml_path, TomlVersion::default()).unwrap();
        let Some((_, tombi_document_tree::Value::Table(dependencies))) =
            dig_keys(&document_tree, &["dependencies"])
        else {
            panic!("dependencies table not found");
        };
        let (crate_key, crate_value) = dependencies
            .key_values()
            .into_iter()
            .next()
            .expect("dependency entry not found");
        let features = disabled_cargo_toml_link_features();

        let document_links = document_link_for_dependency(
            crate_key,
            crate_value,
            &source_cargo_toml_path,
            &RegistryMap::default(),
            TomlVersion::default(),
            Some(&features),
        )
        .unwrap();

        assert_eq!(document_links.len(), 1);
        assert_eq!(
            document_links[0].tooltip,
            std::borrow::Cow::Borrowed("Open Path File")
        );
    }

    #[test]
    fn path_dependency_skips_cargo_toml_links_when_path_disabled() {
        let temp_dir = tempfile::tempdir().unwrap();
        let source_cargo_toml_path = temp_dir.path().join("Cargo.toml");
        let member_dir = temp_dir.path().join("member");
        let member_cargo_toml_path = member_dir.join("Cargo.toml");

        fs::create_dir_all(&member_dir).unwrap();
        fs::write(
            &source_cargo_toml_path,
            r#"
[package]
name = "app"
version = "0.1.0"

[dependencies]
member = { path = "member" }
"#,
        )
        .unwrap();
        fs::write(
            &member_cargo_toml_path,
            r#"
[package]
name = "member"
version = "0.1.0"
"#,
        )
        .unwrap();

        let (_, document_tree) =
            crate::load_cargo_toml(&source_cargo_toml_path, TomlVersion::default()).unwrap();
        let Some((_, tombi_document_tree::Value::Table(dependencies))) =
            dig_keys(&document_tree, &["dependencies"])
        else {
            panic!("dependencies table not found");
        };
        let (crate_key, crate_value) = dependencies
            .key_values()
            .into_iter()
            .next()
            .expect("dependency entry not found");
        let features = disabled_path_link_features();

        let document_links = document_link_for_dependency(
            crate_key,
            crate_value,
            &source_cargo_toml_path,
            &RegistryMap::default(),
            TomlVersion::default(),
            Some(&features),
        )
        .unwrap();

        assert!(document_links.is_empty());
    }

    #[test]
    fn path_dependency_key_link_is_hidden_when_cargo_toml_disabled() {
        let features = disabled_cargo_toml_link_features();

        let tooltip = dependency_key_tooltip(
            std::borrow::Cow::Borrowed("Open Path File"),
            Some(&features),
        );

        assert!(tooltip.is_none());
    }

    fn default_link_features() -> tombi_config::CargoExtensionFeatures {
        tombi_config::CargoExtensionFeatures::Features(tombi_config::CargoExtensionFeatureTree {
            lsp: Some(tombi_config::CargoLspFeatures::Features(
                tombi_config::CargoLspFeatureTree {
                    completion: None,
                    inlay_hint: None,
                    goto_definition: None,
                    goto_declaration: None,
                    document_link: Some(tombi_config::CargoDocumentLinkFeatures::Features(
                        tombi_config::CargoDocumentLinkFeatureTree {
                            cargo_toml: None,
                            workspace: None,
                            git: None,
                            path: None,
                            crates_io: None,
                        },
                    )),
                    hover: None,
                    code_action: None,
                },
            )),
        })
    }

    #[test]
    fn default_features_disable_cargo_toml_document_link() {
        let features = default_link_features();
        assert!(!cargo_toml_document_link_enabled(Some(&features)));
    }

    #[test]
    fn default_features_disable_git_document_link() {
        let features = default_link_features();
        assert!(!git_document_link_enabled(Some(&features)));
    }

    #[test]
    fn default_features_disable_path_document_link() {
        let features = default_link_features();
        assert!(!path_document_link_enabled(Some(&features)));
    }

    #[test]
    fn default_features_disable_workspace_document_link() {
        let features = default_link_features();
        assert!(!workspace_document_link_enabled(Some(&features)));
    }

    #[test]
    fn default_features_keep_crates_io_document_link_enabled() {
        let features = default_link_features();
        assert!(crates_io_document_link_enabled(Some(&features)));
    }

    fn disabled_git_link_features() -> tombi_config::CargoExtensionFeatures {
        tombi_config::CargoExtensionFeatures::Features(tombi_config::CargoExtensionFeatureTree {
            lsp: Some(tombi_config::CargoLspFeatures::Features(
                tombi_config::CargoLspFeatureTree {
                    completion: None,
                    inlay_hint: None,
                    goto_definition: None,
                    goto_declaration: None,
                    document_link: Some(tombi_config::CargoDocumentLinkFeatures::Features(
                        tombi_config::CargoDocumentLinkFeatureTree {
                            cargo_toml: None,
                            workspace: None,
                            git: Some(tombi_config::ToggleFeatureDefaultFalse {
                                enabled: Some(false.into()),
                            }),
                            path: None,
                            crates_io: None,
                        },
                    )),
                    hover: None,
                    code_action: None,
                },
            )),
        })
    }

    #[test]
    fn git_document_link_disabled_by_setting() {
        let features = disabled_git_link_features();
        assert!(!git_document_link_enabled(Some(&features)));
    }
}
