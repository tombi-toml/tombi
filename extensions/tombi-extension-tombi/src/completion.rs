use tombi_config::{DOT_TOMBI_TOML_FILENAME, TOMBI_TOML_FILENAME, TomlVersion};
use tombi_extension::{CommentContext, CompletionContent, CompletionHint, completion_file_path};
use tombi_schema_store::{Accessor, matches_accessors};

pub async fn completion(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
    _toml_version: TomlVersion,
    _completion_hint: Option<CompletionHint>,
    comment_context: Option<&CommentContext>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if comment_context.is_some() {
        return Ok(None);
    }

    let path = text_document_uri.path();
    if !(path.ends_with(DOT_TOMBI_TOML_FILENAME) || path.ends_with(TOMBI_TOML_FILENAME)) {
        return Ok(None);
    }

    if matches_accessors!(accessors, ["schema", "catalog", "path"])
        || matches_accessors!(accessors, ["schemas", _, "path"])
    {
        if let Some(completions) = completion_file_path(
            text_document_uri,
            document_tree,
            position,
            accessors,
            Some(&["json"]),
        ) {
            return Ok(Some(completions));
        }
    }

    Ok(None)
}
