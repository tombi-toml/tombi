use std::borrow::Cow;

use tombi_config::TomlVersion;
use tombi_document_tree::dig_keys;
use tower_lsp::lsp_types::{TextDocumentIdentifier, Url};

use crate::{find_member_project_toml, find_workspace_pyproject_toml, goto_member_pyprojects};

pub enum DocumentLinkToolTip {
    PyprojectToml,
    PyprojectTomlFirstMember,
    WorkspacePyprojectToml,
}

impl From<&DocumentLinkToolTip> for &'static str {
    #[inline]
    fn from(val: &DocumentLinkToolTip) -> Self {
        match val {
            DocumentLinkToolTip::PyprojectToml => "Open pyproject.toml",
            DocumentLinkToolTip::PyprojectTomlFirstMember => "Open first pyproject.toml in members",
            DocumentLinkToolTip::WorkspacePyprojectToml => "Open Workspace pyproject.toml",
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
    if !text_document.uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    let Some(pyproject_toml_path) = text_document.uri.to_file_path().ok() else {
        return Ok(None);
    };

    let mut document_links = vec![];

    if let Some((_, tombi_document_tree::Value::Table(workspace))) =
        dig_keys(document_tree, &["tool", "uv", "workspace"])
    {
        document_links.extend(document_link_for_workspace_pyproject_toml(
            document_tree,
            workspace,
            &pyproject_toml_path,
            toml_version,
        )?);
    } else if let Some((_, tombi_document_tree::Value::Table(sources))) =
        dig_keys(document_tree, &["tool", "uv", "sources"])
    {
        for (package_name_key, source) in sources.key_values() {
            document_links.extend(document_link_for_member_pyproject_toml(
                package_name_key,
                source,
                &pyproject_toml_path,
                toml_version,
            )?);
        }
    }

    if document_links.is_empty() {
        return Ok(None);
    }

    Ok(Some(document_links))
}

fn document_link_for_workspace_pyproject_toml(
    workspace_document_tree: &tombi_document_tree::DocumentTree,
    workspace: &tombi_document_tree::Table,
    workspace_pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let Some(tombi_document_tree::Value::Array(members)) = workspace.get("members") else {
        return Ok(Vec::with_capacity(0));
    };

    let mut total_document_links = vec![];
    for (i, member) in members.values().iter().enumerate() {
        let tombi_document_tree::Value::String(member) = member else {
            continue;
        };

        let Ok(member_paskage_locations) = goto_member_pyprojects(
            workspace_document_tree,
            &[
                tombi_schema_store::Accessor::Key("tool".to_string()),
                tombi_schema_store::Accessor::Key("uv".to_string()),
                tombi_schema_store::Accessor::Key("workspace".to_string()),
                tombi_schema_store::Accessor::Key("members".to_string()),
                tombi_schema_store::Accessor::Index(i),
            ],
            workspace_pyproject_toml_path,
            toml_version,
        ) else {
            continue;
        };

        let mut member_document_links =
            member_paskage_locations.into_iter().filter_map(|location| {
                let Ok(member_pyproject_toml_url) =
                    Url::from_file_path(&location.pyproject_toml_path)
                else {
                    return None;
                };
                Some(tombi_extension::DocumentLink {
                    target: member_pyproject_toml_url,
                    range: member.unquoted_range(),
                    tooltip: DocumentLinkToolTip::PyprojectTomlFirstMember.into(),
                })
            });

        match member_document_links.size_hint() {
            (_, Some(n)) if n > 0 => {
                if let Some(mut document_link) = member_document_links.next() {
                    if n == 1 {
                        document_link.tooltip = DocumentLinkToolTip::PyprojectToml.into();
                    }
                    total_document_links.push(document_link);
                }
            }
            _ => {}
        }
    }

    Ok(total_document_links)
}

fn document_link_for_member_pyproject_toml(
    package_name_key: &tombi_document_tree::Key,
    source: &tombi_document_tree::Value,
    pyproject_toml_path: &std::path::Path,
    toml_version: TomlVersion,
) -> Result<Vec<tombi_extension::DocumentLink>, tower_lsp::jsonrpc::Error> {
    let tombi_document_tree::Value::Table(source) = source else {
        return Ok(Vec::with_capacity(0));
    };

    let Some((workspace_pyproject_toml_path, workspace_pyproject_toml_document_tree)) =
        find_workspace_pyproject_toml(pyproject_toml_path, toml_version)
    else {
        return Ok(Vec::with_capacity(0));
    };

    let Ok(workspace_pyproject_toml_url) = Url::from_file_path(&workspace_pyproject_toml_path)
    else {
        return Ok(Vec::with_capacity(0));
    };

    let mut document_links = vec![];
    if let Some((workspace_key, tombi_document_tree::Value::Boolean(is_workspace))) =
        source.get_key_value("workspace")
    {
        if is_workspace.value() {
            if let Some((member_project_toml_path, _)) = find_member_project_toml(
                package_name_key.value(),
                &workspace_pyproject_toml_document_tree,
                &workspace_pyproject_toml_path,
                toml_version,
            ) {
                if let Ok(member_project_toml_url) = Url::from_file_path(&member_project_toml_path)
                {
                    document_links.push(tombi_extension::DocumentLink {
                        target: member_project_toml_url,
                        range: package_name_key.unquoted_range(),
                        tooltip: DocumentLinkToolTip::PyprojectToml.into(),
                    });
                }
                document_links.push(tombi_extension::DocumentLink {
                    target: workspace_pyproject_toml_url.clone(),
                    range: workspace_key.range() + is_workspace.range(),
                    tooltip: DocumentLinkToolTip::WorkspacePyprojectToml.into(),
                });
            }
        }
    }

    Ok(document_links)
}
