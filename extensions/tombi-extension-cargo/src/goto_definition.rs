use crate::{goto_definition_for_crate_cargo_toml, goto_definition_for_workspace_cargo_toml};
use tombi_config::TomlVersion;
use tower_lsp::lsp_types::TextDocumentIdentifier;

pub async fn goto_definition(
    text_document: &TextDocumentIdentifier,
    document_tree: &tombi_document_tree::DocumentTree,
    accessors: &[tombi_schema_store::Accessor],
    toml_version: TomlVersion,
) -> Result<Option<Vec<tombi_extension::DefinitionLocation>>, tower_lsp::jsonrpc::Error> {
    // Check if current file is Cargo.toml
    if !text_document.uri.path().ends_with("Cargo.toml") {
        return Ok(Default::default());
    }
    let Ok(cargo_toml_path) = text_document.uri.to_file_path() else {
        return Ok(Default::default());
    };

    let locations =
        if accessors.first() == Some(&tombi_schema_store::Accessor::Key("workspace".to_string())) {
            itertools::concat([
                goto_definition_for_workspace_cargo_toml(
                    document_tree,
                    accessors,
                    &cargo_toml_path,
                    toml_version,
                    true,
                )?,
                // For Root Package
                // See: https://doc.rust-lang.org/cargo/reference/workspaces.html#root-package
                goto_definition_for_crate_cargo_toml(
                    document_tree,
                    accessors,
                    &cargo_toml_path,
                    toml_version,
                    true,
                )?,
            ])
        } else {
            goto_definition_for_crate_cargo_toml(
                document_tree,
                accessors,
                &cargo_toml_path,
                toml_version,
                true,
            )?
        };

    if locations.is_empty() {
        return Ok(None);
    }

    Ok(Some(locations))
}
