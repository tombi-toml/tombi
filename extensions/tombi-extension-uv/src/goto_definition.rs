use tombi_config::TomlVersion;
use tombi_extension::DefinitionLocations;
use tombi_schema_store::match_accessors;
use tower_lsp::lsp_types::TextDocumentIdentifier;

use crate::{
    goto_definition_for_member_pyproject_toml, goto_definition_for_workspace_pyproject_toml,
};

pub async fn goto_definition(
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
) -> Result<Option<tombi_extension::DefinitionLocations>, tower_lsp::jsonrpc::Error> {
    // Check if current file is pyproject.toml
    if !text_document.uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }
    let Ok(pyproject_toml_path) = text_document.uri.to_file_path() else {
        return Ok(None);
    };

    let locations = if match_accessors!(accessors[..3], ["tool", "uv", "sources"]) {
        goto_definition_for_member_pyproject_toml(
            document_tree,
            accessors,
            &pyproject_toml_path,
            toml_version,
            true,
        )?
    } else if match_accessors!(accessors[..3], ["tool", "uv", "workspace"]) {
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

    Ok(Some(DefinitionLocations(locations)))
}
