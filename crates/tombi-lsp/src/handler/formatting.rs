use itertools::{Either, Itertools};
use tombi_config::FormatOptions;
use tombi_glob::{matches_file_patterns, MatchResult};
use tombi_text::{IntoLsp, Position, Range};
use tower_lsp::lsp_types::{
    notification::PublishDiagnostics, DocumentFormattingParams, PublishDiagnosticsParams, TextEdit,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{backend::Backend, config_manager::ConfigSchemaStore};

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_formatting(
    backend: &Backend,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_formatting");
    tracing::trace!(?params);

    let DocumentFormattingParams { text_document, .. } = params;
    let text_document_uri = text_document.uri.into();

    let ConfigSchemaStore {
        config,
        schema_store,
        config_path,
    } = backend
        .config_manager
        .config_schema_store_for_uri(&text_document_uri)
        .await;

    if !config
        .lsp()
        .and_then(|server| server.formatting.as_ref())
        .and_then(|formatting| formatting.enabled)
        .unwrap_or_default()
        .value()
    {
        tracing::debug!("`server.formatting.enabled` is false");
        return Ok(None);
    }

    if let Ok(text_document_path) = text_document_uri.to_file_path() {
        match matches_file_patterns(&text_document_path, config_path.as_deref(), &config) {
            MatchResult::Matched => {}
            MatchResult::IncludeNotMatched => {
                tracing::info!(
                    "Skip {text_document_path:?} because it is not in config.files.include"
                );
                return Ok(None);
            }
            MatchResult::ExcludeMatched => {
                tracing::info!("Skip {text_document_path:?} because it is in config.files.exclude");
                return Ok(None);
            }
        }
    }

    let mut document_sources = backend.document_sources.write().await;
    let Some(document_source) = document_sources.get_mut(&text_document_uri) else {
        return Ok(None);
    };

    let toml_version = document_source.toml_version;

    match tombi_formatter::Formatter::new(
        toml_version,
        config.format.as_ref().unwrap_or(&FormatOptions::default()),
        Some(Either::Left(&text_document_uri)),
        &schema_store,
    )
    .format(document_source.text())
    .await
    {
        Ok(formatted) => {
            if document_source.text() != formatted {
                let edits = compute_text_edits(
                    document_source.text(),
                    &formatted,
                    document_source.line_index(),
                );
                tracing::debug!(?edits);
                document_source.set_text(formatted, toml_version);

                return Ok(Some(edits));
            } else {
                tracing::debug!("no change");
                backend
                    .client
                    .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                        uri: text_document_uri.into(),
                        diagnostics: Vec::with_capacity(0),
                        version: document_source.version,
                    })
                    .await;
            }
        }
        Err(diagnostics) => {
            tracing::error!("Failed to format");
            let line_index = document_source.line_index();
            backend
                .client
                .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: text_document_uri.into(),
                    diagnostics: diagnostics
                        .into_iter()
                        .map(|diagnostic| diagnostic.into_lsp(line_index))
                        .collect_vec(),
                    version: document_source.version,
                })
                .await;
        }
    }

    Ok(None)
}

/// Computes incremental text edits between old and new text
/// Returns a vector of TextEdit objects representing the minimal changes needed
/// Uses a grapheme-aware prefix/suffix diff so edits stay minimal and on character boundaries
fn compute_text_edits(
    old_text: &str,
    new_text: &str,
    line_index: &tombi_text::LineIndex,
) -> Vec<TextEdit> {
    if old_text == new_text {
        return Vec::with_capacity(0);
    }

    let common_prefix_bytes = old_text
        .graphemes(true)
        .zip(new_text.graphemes(true))
        .take_while(|(old_grapheme, new_grapheme)| old_grapheme == new_grapheme)
        .map(|(grapheme, _)| grapheme.len())
        .sum::<usize>();

    let old_suffix = &old_text[common_prefix_bytes..];
    let new_suffix = &new_text[common_prefix_bytes..];

    let common_suffix_bytes = old_suffix
        .graphemes(true)
        .rev()
        .zip(new_suffix.graphemes(true).rev())
        .take_while(|(old_grapheme, new_grapheme)| old_grapheme == new_grapheme)
        .map(|(grapheme, _)| grapheme.len())
        .sum::<usize>()
        .min(old_suffix.len())
        .min(new_suffix.len());

    let change_start = common_prefix_bytes;
    let old_change_end = old_text
        .len()
        .saturating_sub(common_suffix_bytes)
        .max(change_start);
    let new_change_end = new_text
        .len()
        .saturating_sub(common_suffix_bytes)
        .max(change_start);

    if change_start == old_change_end && change_start == new_change_end {
        return Vec::with_capacity(0);
    }

    let start_position = position_at_offset(line_index, change_start);
    let end_position = position_at_offset(line_index, old_change_end);

    let replacement = new_text[change_start..new_change_end].to_string();

    vec![TextEdit {
        range: Range::new(start_position, end_position).into_lsp(line_index),
        new_text: replacement,
    }]
}

fn position_at_offset(line_index: &tombi_text::LineIndex, offset: usize) -> Position {
    if line_index.is_empty() {
        return Position::new(0, 0);
    }

    let mut last_line = 0u32;
    let mut last_text = "";

    for (idx, span) in line_index.iter().enumerate() {
        let line = idx as u32;
        let line_text = line_index.line_text(line).unwrap_or("");

        last_line = line;
        last_text = line_text;

        if offset <= usize::from(span.end) {
            let slice_end = offset
                .saturating_sub(usize::from(span.start))
                .min(line_text.len());
            let column =
                UnicodeSegmentation::graphemes(&line_text[..slice_end], true).count() as u32;
            return Position::new(line, column);
        }
    }

    let column = UnicodeSegmentation::graphemes(last_text, true).count() as u32;
    Position::new(last_line, column)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tombi_text::{EncodingKind, LineIndex, Range};

    #[test]
    fn test_compute_text_edits_no_changes() {
        let old_text = "hello world";
        let new_text = "hello world";
        let line_index = LineIndex::new(old_text, EncodingKind::Utf16);
        let edits = compute_text_edits(old_text, new_text, &line_index);

        pretty_assertions::assert_eq!(edits, vec![]);
    }

    #[test]
    fn test_compute_text_edits_append_final_newline() {
        let old_text = "hello world";
        let new_text = "hello world\n";
        let line_index = LineIndex::new(old_text, EncodingKind::Utf16);
        let edits = compute_text_edits(old_text, new_text, &line_index);

        pretty_assertions::assert_eq!(
            edits,
            vec![TextEdit {
                range: Range::from(((0, 11), (0, 11))).into_lsp(&line_index),
                new_text: "\n".to_string(),
            }]
        );
    }

    #[test]
    fn test_compute_text_edits_no_changes_final_newline() {
        let old_text = "hello world\n";
        let new_text = "hello world\n";
        let line_index = LineIndex::new(old_text, EncodingKind::Utf16);
        let edits = compute_text_edits(old_text, new_text, &line_index);

        pretty_assertions::assert_eq!(edits, vec![]);
    }

    #[test]
    fn test_compute_text_edits_trim_final_newlines() {
        // Test case: remove any final trailing newlines leaving a single newline
        let old_text = "line1\n\n\n";
        let new_text = "line1\n";
        let line_index = LineIndex::new(old_text, EncodingKind::Utf16);
        let edits = compute_text_edits(old_text, new_text, &line_index);

        pretty_assertions::assert_eq!(
            edits,
            vec![TextEdit {
                range: Range::from(((1, 0), (3, 0))).into_lsp(&line_index),
                new_text: "".to_string(),
            }]
        );
    }

    #[test]
    fn test_compute_text_edits_simple_replacement() {
        let old_text = "hello world";
        let new_text = "hello universe";
        let line_index = LineIndex::new(old_text, EncodingKind::Utf16);
        let edits = compute_text_edits(old_text, new_text, &line_index);

        pretty_assertions::assert_eq!(
            edits,
            vec![TextEdit {
                range: Range::from(((0, 6), (0, 11))).into_lsp(&line_index),
                new_text: "universe".to_string(),
            }]
        );
    }

    #[test]
    fn test_compute_text_edits_multiline() {
        let old_text = "line1\nline2\nline3";
        let new_text = "line1\nmodified line2\nline3";
        let line_index = LineIndex::new(old_text, EncodingKind::Utf16);
        let edits = compute_text_edits(old_text, new_text, &line_index);

        pretty_assertions::assert_eq!(
            edits,
            vec![TextEdit {
                range: Range::from(((1, 0), (1, 0))).into_lsp(&line_index),
                new_text: "modified ".to_string(),
            }]
        );
    }
}
