use tombi_document_tree_syntax::{Value, dig_accessors};
use tombi_extension::{
    CompletionContent, CompletionContentPriority, CompletionEdit, CompletionHint, CompletionKind,
    CompletionTextEdit, InsertTextFormat, TextEdit,
};
use tombi_schema_store::Accessor;

use crate::{
    channels::query_scope,
    conda::fetch_package_versions,
    dependency::{channel_dependency_source, find_conda_dependency},
    is_pixi_toml,
};

pub async fn completion(
    text_document_uri: &tombi_uri::Uri,
    document_tree: &tombi_document_tree_syntax::DocumentTree<'_>,
    offset: tombi_text::Offset,
    accessors: &[Accessor],
    completion_hint: Option<CompletionHint>,
    in_comment: bool,
    offline: bool,
    cache_options: Option<&tombi_cache::Options>,
    features: Option<&tombi_config::PixiExtensionFeatures>,
) -> Result<Option<Vec<CompletionContent>>, tower_lsp_server::jsonrpc::Error> {
    if !is_pixi_toml(text_document_uri) || in_comment {
        return Ok(None);
    }

    if !dependency_version_completion_enabled(features) {
        return Ok(None);
    }

    let Some(dependency) = find_conda_dependency(accessors) else {
        return Ok(None);
    };

    let version_value = match dig_accessors(document_tree, accessors) {
        Some((_, value @ (Value::String(_) | Value::Incomplete { .. }))) => value,
        _ => return Ok(None),
    };

    let Some(inline_channel) = channel_dependency_source(document_tree, &dependency) else {
        return Ok(None);
    };

    let Some(manifest_dir) = text_document_uri
        .to_file_path()
        .ok()
        .and_then(|path| path.parent().map(ToOwned::to_owned))
    else {
        return Ok(None);
    };

    let scope = query_scope(document_tree, &dependency, inline_channel, &manifest_dir);
    let Some(versions) =
        fetch_package_versions(dependency.package_name, &scope, offline, cache_options).await
    else {
        return Ok(None);
    };

    let items = versions
        .into_iter()
        .take(100)
        .enumerate()
        .map(|(i, version)| CompletionContent {
            label: format!("\"{version}\""),
            kind: CompletionKind::Enum,
            emoji_icon: Some('📦'),
            priority: CompletionContentPriority::Custom(format!("10__pixi_{i:>03}__")),
            detail: Some("Conda package version".to_string()),
            documentation: None,
            filter_text: None,
            schema_base_uri: None,
            deprecated: None,
            edit: match version_value {
                Value::String(value_string) => CompletionEdit::new_string_literal_while_editing(
                    &format!("\"{version}\""),
                    value_string.span(),
                ),
                Value::Incomplete { .. } => Some(CompletionEdit {
                    text_edit: CompletionTextEdit::Edit(TextEdit {
                        span: tombi_text::Span::empty(offset),
                        new_text: format!(" = \"{version}\""),
                    }),
                    insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                    additional_text_edits: match completion_hint {
                        Some(
                            CompletionHint::DotTrigger { span, .. }
                            | CompletionHint::EqualTrigger { span, .. },
                        ) => Some(vec![TextEdit {
                            span,
                            new_text: "".to_string(),
                        }]),
                        _ => None,
                    },
                }),
                _ => None,
            },
            preselect: (i == 0).then_some(true),
            in_comment: false,
        })
        .collect();

    Ok(Some(items))
}

fn dependency_version_completion_enabled(
    features: Option<&tombi_config::PixiExtensionFeatures>,
) -> bool {
    let Some(completion) = features
        .and_then(|features| features.lsp())
        .and_then(|lsp| lsp.completion())
    else {
        return true;
    };

    completion.enabled().value()
        && completion
            .dependency_version()
            .map(|feature| feature.enabled())
            .unwrap_or_default()
            .value()
}
