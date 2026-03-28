use tombi_config::TomlVersion;
use tombi_extension::{
    CommentContext, CompletionContent, CompletionHint, completion_directory_path,
    completion_file_path,
};
use tombi_schema_store::{Accessor, matches_accessors};

enum PyprojectCompletionFeature {
    Path,
}

pub async fn completion(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
    _toml_version: TomlVersion,
    _completion_hint: Option<CompletionHint>,
    comment_context: Option<&CommentContext>,
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp::jsonrpc::Error> {
    if comment_context.is_some() {
        return Ok(None);
    }

    if !text_document_uri.path().ends_with("pyproject.toml") {
        return Ok(None);
    }

    if !pyproject_completion_root_enabled(features) {
        return Ok(None);
    }

    if let Some(completions) =
        pyproject_completion_enabled(features, PyprojectCompletionFeature::Path)
            .then(|| {
                completion_pyproject_file_path(
                    text_document_uri,
                    document_tree,
                    position,
                    accessors,
                )
            })
            .flatten()
    {
        return Ok(Some(completions));
    }

    Ok(None)
}

fn pyproject_completion_root_enabled(
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
) -> bool {
    features.map_or(
        true,
        tombi_config::PyprojectExtensionFeatures::completion_enabled,
    )
}

fn pyproject_completion_enabled(
    features: Option<&tombi_config::PyprojectExtensionFeatures>,
    feature: PyprojectCompletionFeature,
) -> bool {
    features.map_or(true, |features| match feature {
        PyprojectCompletionFeature::Path => features.path_completion_enabled(),
    })
}

fn completion_pyproject_file_path(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree::DocumentTree,
    position: tombi_text::Position,
    accessors: &[Accessor],
) -> Option<Vec<CompletionContent>> {
    // Pyproject workspace: directory paths only (members, exclude)
    if matches_accessors!(accessors, ["tool", "uv", "workspace", "members", _])
        || matches_accessors!(accessors, ["tool", "uv", "workspace", "exclude", _])
    {
        return completion_directory_path(text_document_uri, document_tree, position, accessors);
    }

    // Pyproject sources: path to local package (file or directory)
    if matches_accessors!(accessors, ["tool", "uv", "sources", _, "path"]) {
        return completion_file_path(
            text_document_uri,
            document_tree,
            position,
            accessors,
            Some(&[]),
        );
    }

    // Pyproject standard: build-system, readme, license
    if matches_accessors!(accessors, ["build-system", "backend-path", _])
        || matches_accessors!(accessors, ["project", "readme"])
        || matches_accessors!(accessors, ["project", "readme", "file"])
        || matches_accessors!(accessors, ["project", "license", "file"])
        || matches_accessors!(accessors, ["project", "license-files", _])
    {
        return completion_file_path(
            text_document_uri,
            document_tree,
            position,
            accessors,
            Some(&[]),
        );
    }

    None
}
