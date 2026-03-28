use tombi_config::TomlVersion;
use tombi_extension::HoverMetadata;

pub async fn hover(
    _text_document_uri: &tombi_uri::Uri,
    _document_tree: &tombi_document_tree::DocumentTree,
    _accessors: &[tombi_schema_store::Accessor],
    _toml_version: TomlVersion,
    _offline: bool,
) -> Result<Option<HoverMetadata>, tower_lsp::jsonrpc::Error> {
    Ok(None)
}
