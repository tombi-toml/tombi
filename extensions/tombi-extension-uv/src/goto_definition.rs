use tombi_config::TomlVersion;
use tombi_schema_store::matches_accessors;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::{
    goto_definition_for_member_pyproject_toml, goto_definition_for_workspace_pyproject_toml,
};

pub async fn goto_definition(
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
) -> Result<Option<Vec<tombi_extension::DefinitionLocation>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is pyproject.toml
    if !text_document.uri.path().ends_with("pyproject.toml") {
        return Ok(Default::default());
    }
    let Ok(pyproject_toml_path) = text_document.uri.to_file_path() else {
        return Ok(Default::default());
    };

    let locations = if matches_accessors!(
        accessors[..accessors.len().min(3)],
        ["tool", "uv", "sources"]
    ) {
        goto_definition_for_member_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
            accessors.last() != Some(&tombi_schema_store::Accessor::Key("workspace".to_string())),
        )?
    } else if matches_accessors!(
        accessors[..accessors.len().min(3)],
        ["tool", "uv", "workspace"]
    ) {
        goto_definition_for_workspace_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
        )?
    } else {
        Vec::with_capacity(0)
    };

    if locations.is_empty() {
        return Ok(None);
    }

    Ok(Some(locations))
}
