use tombi_config::{DOT_TOMBI_TOML_FILENAME, TOMBI_TOML_FILENAME, TomlVersion, config_base_dir};
use tombi_extension::{
    CommentContext, CompletionContent, CompletionHint, completion_file_path_from_base_dir,
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
    features: Option<&tombi_config::TombiExtensionFeatures>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if comment_context.is_some() {
        return Ok(None);
    }

    if !features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.completion())
        .map(|completion| completion.enabled())
        .unwrap_or_default()
        .value()
    {
        return Ok(None);
    }

    let Some(text_document_path) = text_document_uri.to_file_path().ok() else {
        return Ok(None);
    };

    if !matches!(
        text_document_path
            .file_name()
            .and_then(|name| name.to_str()),
        Some(DOT_TOMBI_TOML_FILENAME | TOMBI_TOML_FILENAME)
    ) {
        return Ok(None);
    }

    let Some(base_dir) = config_base_dir(&text_document_path) else {
        return Ok(None);
    };

    if features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.completion())
        .and_then(|completion| completion.path())
        .map(|path| path.enabled())
        .unwrap_or_default()
        .value()
    {
        if matches_accessors!(accessors, ["schema", "catalog", "paths", _])
            && let Some(completions) = completion_file_path_from_base_dir(
                &base_dir,
                document_tree,
                position,
                accessors,
                Some(&["json"]),
            )
        {
            return Ok(Some(completions));
        }

        if matches_accessors!(accessors, ["schemas", _, "path"])
            && let Some(completions) = completion_file_path_from_base_dir(
                &base_dir,
                document_tree,
                position,
                accessors,
                Some(&["json"]),
            )
        {
            return Ok(Some(completions));
        }
    }

    Ok(None)
}
