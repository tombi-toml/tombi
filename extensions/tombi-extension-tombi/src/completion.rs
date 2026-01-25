use tombi_config::TomlVersion;
use tombi_document_tree::dig_accessors;
use tombi_extension::{
    CommentContext, CompletionContent, CompletionHint, get_file_path_completions,
};
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

    if !text_document_uri.path().ends_with("tombi.toml") {
        return Ok(None);
    }

    if (matches_accessors!(accessors, ["schema", "catalog", "path"])
        || matches_accessors!(accessors, ["schemas", _, "path"]))
        && let Some(completions) =
            completion_schema_file_path(text_document_uri, document_tree, position, accessors)
    {
        return Ok(Some(completions));
    }

    Ok(None)
}

fn completion_schema_file_path(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
) -> Option<Vec<CompletionContent>> {
    let Ok(source_path) = text_document_uri.to_file_path() else {
        return None;
    };
    let Some(base_dir) = source_path.parent() else {
        return None;
    };

    let Some((_, tombi_document_tree::Value::String(string))) =
        dig_accessors(document_tree, accessors)
    else {
        return None;
    };

    if !string.range().contains(position) {
        return None;
    }

    let completions =
        get_file_path_completions(base_dir, string.value(), string.unquoted_range(), &["json"]);

    if completions.is_empty() {
        None
    } else {
        Some(completions)
    }
}
