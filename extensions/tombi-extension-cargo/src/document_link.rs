use std::borrow::Cow;

use crate::{
    find_package_cargo_toml_paths, find_path_crate_cargo_toml, find_workspace_cargo_toml,
    get_workspace_path, load_cargo_toml,
};
use itertools::Itertools;
use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;
use tower_lsp::lsp_types::{TextDocumentIdentifier, Url};

type RegistoryMap = ahash::AHashMap<String, Registory>;
const DEFAULT_REGISTORY_INDEX: &str = "https://crates.io/crates";

struct Registory {
    index: String,
}

pub enum DocumentLinkToolTip {
    GitRepository,
    Registory,
    CrateIo,
    CargoToml,
    CargoTomlFirstMember,
    WorkspaceCargoToml,
}

impl From<&DocumentLinkToolTip> for &'static str {
    #[inline]
    fn from(val: &DocumentLinkToolTip) -> Self {
        match val {
            DocumentLinkToolTip::GitRepository => "Open Git Repository",
            DocumentLinkToolTip::Registory => "Open Registry",
            DocumentLinkToolTip::CrateIo => "Open crate.io",
            DocumentLinkToolTip::CargoToml => "Open Cargo.toml",
            DocumentLinkToolTip::CargoTomlFirstMember => "Open first Cargo.toml in members",
            DocumentLinkToolTip::WorkspaceCargoToml => "Open Workspace Cargo.toml",
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
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    toml_version: TomlVersion,
) -> Result<Option<Vec<tombi_extension::DocumentLink>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is Cargo.toml
    if !text_document.uri.path().ends_with("Cargo.toml") {
        return Ok(None);
    }
    let Some(cargo_toml_path) = text_document.uri.to_file_path().ok() else {
        return Ok(None);
    };

    let mut document_links = vec![];

    if document_tree.contains_key("workspace") {
        document_links.extend(document_link_for_workspace_cargo_toml(
            document_tree,
            &cargo_toml_path,
            toml_version,
        )?);

        // For Root Package
        // See: https://doc.rust-lang.org/cargo/reference/workspaces.html#root-package
        if document_tree.contains_key("package") {
            document_links.extend(document_link_for_crate_cargo_toml(
                document_tree,
                &cargo_toml_path,
                toml_version,
            )?);
        }
    } else {
        document_links.extend(document_link_for_crate_cargo_toml(
            document_tree,
            &cargo_toml_path,
            toml_version,
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
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut total_document_links = vec![];

    let registories = get_registories(workspace_cargo_toml_path, toml_version).unwrap_or_default();

    if let Some((_, tombi_document_tree::Value::Table(dependencies))) =
        dig_keys(workspace_document_tree, &["workspace", "dependencies"])
    {
        total_document_links.extend(document_link_for_workspace_depencencies(
            dependencies,
            workspace_cargo_toml_path,
            &registories,
            toml_version,
        )?);
    }
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
            _ => Vec::new(),
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
                    let cargo_toml_document_tree = load_cargo_toml(&cargo_toml_path, toml_version)?;
                    let (_, package_name) =
                        dig_keys(&cargo_toml_document_tree, &["package", "name"])?;
                    let package_name = match package_name {
                        tombi_document_tree::Value::String(s) => s,
                        _ => return None,
                    };

                    let mut target = Url::from_file_path(&cargo_toml_path).ok()?;
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
    registories: &RegistoryMap,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut total_document_links = vec![];
    for (crate_name, crate_value) in dependencies.key_values() {
        if let Ok(document_links) = document_link_for_workspace_dependency(
            crate_name,
            crate_value,
            workspace_cargo_toml_path,
            registories,
            toml_version,
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
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut total_dependencies = vec![];
    for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some((_, tombi_document_tree::Value::Table(dependencies))) =
            dig_keys(crate_document_tree, &[key])
        {
            total_dependencies.extend(dependencies.key_values());
        }
    }

    let mut total_document_links = vec![];
    if let Some((workspace_cargo_toml_path, workspace_document_tree)) = find_workspace_cargo_toml(
        crate_cargo_toml_path,
        get_workspace_path(crate_document_tree),
        toml_version,
    ) {
        let registories =
            get_registories(&workspace_cargo_toml_path, toml_version).unwrap_or_default();

        // Support Workspace
        // See: https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field
        if let Some((_, tombi_document_tree::Value::String(workspace_path))) =
            dig_keys(crate_document_tree, &["package", "workspace"])
        {
            if let Ok(target) = Url::from_file_path(&workspace_cargo_toml_path) {
                total_document_links.push(tombi_extension::DocumentLink {
                    target,
                    range: workspace_path.unquoted_range(),
                    tooltip: DocumentLinkToolTip::WorkspaceCargoToml.into(),
                });
            }
        }

        // Support Package Table
        // See: https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table
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
                let Ok(mut target) = Url::from_file_path(&workspace_cargo_toml_path) else {
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

        // Support Lints Workspace
        // See: https://doc.rust-lang.org/cargo/reference/workspaces.html#the-lints-table
        if let (
            Some((workspace_key, tombi_document_tree::Value::Boolean(value))),
            Some((workspace_lints_key, _)),
        ) = (
            dig_keys(crate_document_tree, &["lints", "workspace"]),
            dig_keys(&workspace_document_tree, &["workspace", "lints"]),
        ) {
            if let Ok(mut target) = Url::from_file_path(&workspace_cargo_toml_path) {
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
        }

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
                &registories,
                toml_version,
            ) {
                total_document_links.extend(document_links);
            }
        }
    } else {
        let registories = get_registories(crate_cargo_toml_path, toml_version).unwrap_or_default();

        for (crate_key, crate_value) in total_dependencies {
            if let Ok(document_links) = document_link_for_dependency(
                crate_key,
                crate_value,
                crate_cargo_toml_path,
                &registories,
                toml_version,
            ) {
                total_document_links.extend(document_links);
            }
        }
    }

    Ok(total_document_links)
}

fn document_link_for_workspace_dependency(
    crate_key: &tombi_document_tree::Key,
    crate_value: &tombi_document_tree::Value,
    workspace_cargo_toml_path: &std::path::Path,
    registories: &RegistoryMap,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    match document_link_for_dependency(
        crate_key,
        crate_value,
        workspace_cargo_toml_path,
        registories,
        toml_version,
    )? {
        Some(document_link) => Ok(vec![
            tombi_extension::DocumentLink {
                target: document_link.target.clone(),
                range: crate_key.unquoted_range(),
                tooltip: document_link.tooltip.clone(),
            },
            document_link,
        ]),
        None => Ok(get_crate_io_crate_link(crate_key, crate_value)
            .into_iter()
            .collect_vec()),
    }
}

fn document_link_for_crate_dependency_has_workspace(
    crate_key: &tombi_document_tree::Key,
    crate_value: &tombi_document_tree::Value,
    crate_cargo_toml_path: &std::path::Path,
    workspace_dependencies: Option<&tombi_document_tree::Table>,
    workspace_cargo_toml_path: &std::path::Path,
    registories: &RegistoryMap,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    match document_link_for_dependency(
        crate_key,
        crate_value,
        crate_cargo_toml_path,
        registories,
        toml_version,
    )? {
        Some(document_link) => Ok(vec![
            tombi_extension::DocumentLink {
                target: document_link.target.clone(),
                range: crate_key.unquoted_range(),
                tooltip: document_link.tooltip.clone(),
            },
            document_link,
        ]),
        None => {
            if let (tombi_document_tree::Value::Table(table), Some(workspace_dependencies)) =
                (crate_value, workspace_dependencies)
            {
                if let Some((workspace_key, tombi_document_tree::Value::Boolean(is_workspace))) =
                    table.get_key_value("workspace")
                {
                    if is_workspace.value() {
                        if let Some(workspace_crate_value) = workspace_dependencies.get(&crate_key)
                        {
                            if let Ok(mut target) = Url::from_file_path(workspace_cargo_toml_path) {
                                let mut document_links = document_link_for_workspace_dependency(
                                    crate_key,
                                    workspace_crate_value,
                                    workspace_cargo_toml_path,
                                    registories,
                                    toml_version,
                                )?
                                .into_iter()
                                .next()
                                .into_iter()
                                .collect_vec();

                                target.set_fragment(Some(&format!(
                                    "L{}",
                                    workspace_crate_value.range().start.line + 1
                                )));
                                document_links.push(tombi_extension::DocumentLink {
                                    target,
                                    range: workspace_key.range() + is_workspace.range(),
                                    tooltip: DocumentLinkToolTip::WorkspaceCargoToml.into(),
                                });

                                return Ok(document_links);
                            }
                        }
                    }
                }
            }

            Ok(get_crate_io_crate_link(crate_key, crate_value)
                .into_iter()
                .collect_vec())
        }
    }
}

fn document_link_for_dependency(
    crate_key: &tombi_document_tree::Key,
    crate_value: &tombi_document_tree::Value,
    crate_cargo_toml_path: &std::path::Path,
    registories: &RegistoryMap,
    toml_version: TomlVersion,
) -> Result<Option<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let mut package_name = crate_key.value();
    if let tombi_document_tree::Value::Table(table) = crate_value {
        if let Some(tombi_document_tree::Value::String(real_package)) = table.get("package") {
            package_name = real_package.value();
        };

        if let Some(tombi_document_tree::Value::String(crate_path)) = table.get("path") {
            if let Some((path_target_cargo_toml_path, path_target_document_tree)) =
                find_path_crate_cargo_toml(
                    crate_cargo_toml_path,
                    std::path::Path::new(crate_path.value()),
                    toml_version,
                )
            {
                if let Some((package_name_key, tombi_document_tree::Value::String(package_name))) =
                    tombi_document_tree::dig_keys(&path_target_document_tree, &["package", "name"])
                {
                    let package_name_check =
                        if let Some(tombi_document_tree::Value::String(real_package_name)) =
                            table.get("package")
                        {
                            real_package_name.value() == crate_key.value()
                        } else {
                            package_name.value() == crate_key.value()
                        };
                    if package_name_check {
                        let Ok(mut target) = Url::from_file_path(path_target_cargo_toml_path)
                        else {
                            return Ok(None);
                        };
                        target.set_fragment(Some(&format!(
                            "L{}",
                            package_name_key.range().start.line + 1
                        )));

                        return Ok(Some(tombi_extension::DocumentLink {
                            target,
                            range: crate_path.unquoted_range(),
                            tooltip: DocumentLinkToolTip::CargoToml.into(),
                        }));
                    }
                }
            }
        }

        if let Some(tombi_document_tree::Value::String(git_url)) = table.get("git") {
            let target = if let Ok(target) = Url::parse(git_url.value()) {
                target
            } else if let Ok(target) = Url::from_file_path(git_url.value()) {
                target
            } else {
                return Ok(None);
            };

            return Ok(Some(tombi_extension::DocumentLink {
                target,
                range: git_url.unquoted_range(),
                tooltip: DocumentLinkToolTip::GitRepository.into(),
            }));
        }

        if let Some(tombi_document_tree::Value::String(registory_name)) = table.get("registory") {
            if let Some(registry) = registories.get(registory_name.value()) {
                if let Ok(target) = Url::parse(&format!("{}/{}", registry.index, package_name)) {
                    return Ok(Some(tombi_extension::DocumentLink {
                        target,
                        range: registory_name.unquoted_range(),
                        tooltip: DocumentLinkToolTip::CrateIo.into(),
                    }));
                }
            }
        }
    }

    Ok(None)
}

fn get_registories(
    workspace_cargo_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<RegistoryMap, tower_lsp::jsonrpc::Error> {
    let mut registories = RegistoryMap::default();
    if let Some(cargo_toml_document_tree) = load_cargo_toml(
        &workspace_cargo_toml_path.join(".cargo/config.toml"),
        toml_version,
    ) {
        if let Some(tombi_document_tree::Value::Table(registories_table)) =
            cargo_toml_document_tree.get("registories")
        {
            for (name, value) in registories_table.key_values() {
                if let tombi_document_tree::Value::Table(table) = value {
                    if let Some(tombi_document_tree::Value::String(index)) = table.get("index") {
                        registories.insert(
                            name.value().into(),
                            Registory {
                                index: index.value().into(),
                            },
                        );
                    }
                }
            }
        }
    }

    Ok(registories)
}

fn get_crate_io_crate_link(
    crate_key: &tombi_document_tree::Key,
    crate_value: &tombi_document_tree::Value,
) -> Option<tombi_extension::DocumentLink> {
    let mut crate_name = crate_key.value();
    if let tombi_document_tree::Value::Table(table) = crate_value {
        if let Some(tombi_document_tree::Value::String(real_package)) = table.get("package") {
            crate_name = real_package.value();
        }
    }

    Url::parse(&format!("{DEFAULT_REGISTORY_INDEX}/{crate_name}"))
        .map(|target| tombi_extension::DocumentLink {
            target,
            range: crate_key.unquoted_range(),
            tooltip: DocumentLinkToolTip::CrateIo.into(),
        })
        .ok()
}
