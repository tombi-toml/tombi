use tombi_extension::{CompletionContent, CompletionHint, completion_file_path_from_base_dir};
use tombi_schema_store::{Accessor, matches_accessors};

use crate::workspace::{config_root, is_nagi_config};

pub async fn completion(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
    _completion_hint: Option<CompletionHint>,
    in_comment: bool,
    features: Option<&tombi_config::NagiSqlExtensionFeatures>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if in_comment || !is_nagi_config(text_document_uri) {
        return Ok(None);
    }

    if !features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.completion())
        .map(|feature| feature.enabled())
        .unwrap_or_default()
        .value()
    {
        return Ok(None);
    }

    let completions = if is_path(accessors) {
        let source_path = text_document_uri.to_file_path().ok();
        let base_dir = source_path.as_deref().and_then(config_root);
        base_dir.and_then(|base_dir| {
            completion_file_path_from_base_dir(
                base_dir,
                document_tree,
                position,
                accessors,
                Some(&[]),
            )
        })
    } else {
        None
    };

    Ok(completions)
}

fn is_path(accessors: &[Accessor]) -> bool {
    matches_accessors!(accessors, ["rules", "include", _])
        || matches_accessors!(accessors, ["rules", "exclude", _])
        || matches_accessors!(accessors, ["workspace", "members", _])
        || matches_accessors!(accessors, ["workspace", "exclude", _])
        || matches_accessors!(accessors, ["sources", _, "dbt", "project-root"])
        || matches_accessors!(accessors, ["sources", _, "manifest", "directory"])
        || matches_accessors!(
            accessors,
            ["workspace", "sources", _, "dbt", "project-root"]
        )
        || matches_accessors!(
            accessors,
            ["workspace", "sources", _, "manifest", "directory"]
        )
        || matches_accessors!(accessors, ["sources", _, "include", _])
        || matches_accessors!(accessors, ["sources", _, "exclude", _])
        || matches_accessors!(accessors, ["sources", _, "migration", "include", _])
        || matches_accessors!(accessors, ["sources", _, "migration", "exclude", _])
        || matches_accessors!(accessors, ["sources", _, "overrides", _, "include", _])
        || matches_accessors!(accessors, ["sources", _, "overrides", _, "exclude", _])
        || matches_accessors!(accessors, ["workspace", "sources", _, "include", _])
        || matches_accessors!(accessors, ["workspace", "sources", _, "exclude", _])
        || matches_accessors!(
            accessors,
            ["workspace", "sources", _, "migration", "include", _]
        )
        || matches_accessors!(
            accessors,
            ["workspace", "sources", _, "migration", "exclude", _]
        )
        || matches_accessors!(
            accessors,
            ["workspace", "sources", _, "overrides", _, "include", _]
        )
        || matches_accessors!(
            accessors,
            ["workspace", "sources", _, "overrides", _, "exclude", _]
        )
}
