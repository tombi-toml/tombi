use itertools::{Either, Itertools};
use tombi_config::FormatOptions;
use tombi_formatter::formatter::definitions::FormatDefinitions;
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

    let Some(root) = backend.get_incomplete_ast(&text_document_uri).await else {
        return Ok(None);
    };

    let source_schema = schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document_uri)))
        .await
        .ok()
        .flatten();

    let tombi_document_comment_directive =
        tombi_validator::comment_directive::get_tombi_document_comment_directive(&root).await;
    let (toml_version, _) = backend
        .source_toml_version(
            tombi_document_comment_directive,
            source_schema.as_ref(),
            &config,
        )
        .await;

    let mut document_sources = backend.document_sources.write().await;
    let Some(document_source) = document_sources.get_mut(&text_document_uri) else {
        return Ok(None);
    };

    let formatter_definitions = FormatDefinitions::default();

    match tombi_formatter::Formatter::new(
        toml_version,
        &formatter_definitions,
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
                    &formatter_definitions,
                    document_source.line_index(),
                );
                tracing::error!(?edits);
                document_source.set_text(formatted);

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
/// Uses a simpler line-based approach to avoid edge cases with complex diffing algorithms
fn compute_text_edits(
    old_text: &str,
    new_text: &str,
    formatter_definitions: &FormatDefinitions,
    line_index: &tombi_text::LineIndex,
) -> Vec<TextEdit> {
    let line_ending = formatter_definitions.line_ending.unwrap_or_default().into();

    let old_lines: Vec<&str> = old_text.split(line_ending).collect();
    let new_lines: Vec<&str> = new_text.split(line_ending).collect();

    // Find common prefix lines
    let common_prefix_lines = old_lines
        .iter()
        .zip(new_lines.iter())
        .take_while(|(a, b)| a == b)
        .count();

    // Find common suffix lines
    let remaining_old = &old_lines[common_prefix_lines..];
    let remaining_new = &new_lines[common_prefix_lines..];

    let common_suffix_lines = remaining_old
        .iter()
        .rev()
        .zip(remaining_new.iter().rev())
        .take_while(|(a, b)| a == b)
        .count();

    // Calculate edit boundaries
    let old_start_line = common_prefix_lines;
    let old_end_line = old_lines.len() - common_suffix_lines;
    let new_start_line = common_prefix_lines;
    let new_end_line = new_lines.len() - common_suffix_lines;

    // If no changes, return empty vector
    if old_start_line >= old_end_line && new_start_line >= new_end_line {
        return Vec::with_capacity(0);
    }

    // Build the replacement text from the changed lines
    let replacement_lines = &new_lines[new_start_line..new_end_line];
    let mut replacement_text = if replacement_lines.is_empty() {
        String::new()
    } else {
        replacement_lines.join(line_ending)
    };

    let end_pos = if old_end_line < old_lines.len() {
        Position::new(old_end_line as u32, 0)
    } else {
        // End of document - need to get the column position
        let last_line = old_lines.last().unwrap_or(&"");
        Position::new(
            old_lines.len().saturating_sub(1) as u32,
            UnicodeSegmentation::graphemes(*last_line, true).count() as u32,
        )
    };

    // Calculate positions
    let start_pos = std::cmp::min(Position::new(old_start_line as u32, 0), end_pos);

    vec![TextEdit {
        range: Range::new(start_pos, end_pos).into_lsp(line_index),
        new_text: replacement_text,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tombi_text::{LineIndex, Position, Range, WideEncoding};

    #[test]
    fn test_compute_text_edits_no_changes() {
        let old_text = "hello world";
        let new_text = "hello world";
        let line_index = LineIndex::new(old_text, WideEncoding::Utf16);
        let edits = compute_text_edits(
            old_text,
            new_text,
            &FormatDefinitions::default(),
            &line_index,
        );

        pretty_assertions::assert_eq!(edits, vec![]);
    }

    #[test]
    fn test_compute_text_edits_append_final_newline() {
        let old_text = "hello world";
        let new_text = "hello world\n";
        let line_index = LineIndex::new(old_text, WideEncoding::Utf16);
        let edits = compute_text_edits(
            old_text,
            new_text,
            &FormatDefinitions::default(),
            &line_index,
        );

        pretty_assertions::assert_eq!(
            edits,
            vec![TextEdit {
                range: Range::from(((1, 0), (1, 0))).into_lsp(&line_index),
                new_text: "".to_string(),
            }]
        );
    }

    #[test]
    fn test_compute_text_edits_no_changes_final_newline() {
        let old_text = "hello world\n";
        let new_text = "hello world\n";
        let line_index = LineIndex::new(old_text, WideEncoding::Utf16);
        let edits = compute_text_edits(
            old_text,
            new_text,
            &FormatDefinitions::default(),
            &line_index,
        );

        pretty_assertions::assert_eq!(edits, vec![]);
    }

    #[test]
    fn test_compute_text_edits_trim_final_newlines() {
        // Test case: remove any final trailing newlines leaving a single newline
        let old_text = "line1\n\n\n";
        let new_text = "line1\n";
        let line_index = LineIndex::new(old_text, WideEncoding::Utf16);
        let edits = compute_text_edits(
            old_text,
            new_text,
            &FormatDefinitions::default(),
            &line_index,
        );

        pretty_assertions::assert_eq!(edits.len(), 1);

        let edit = &edits[0];

        let expected_range: tower_lsp::lsp_types::Range =
            Range::from(((2, 0), (4, 0))).into_lsp(&line_index);

        pretty_assertions::assert_eq!(edit.range, expected_range);

        // Remove the extra newlines with an empty string
        pretty_assertions::assert_eq!(edit.new_text, "");
    }

    #[test]
    fn test_compute_text_edits_simple_replacement() {
        let old_text = "hello world";
        let new_text = "hello universe";
        let line_index = LineIndex::new(old_text, WideEncoding::Utf16);
        let edits = compute_text_edits(
            old_text,
            new_text,
            &FormatDefinitions::default(),
            &line_index,
        );

        pretty_assertions::assert_eq!(edits.len(), 1);
        let edit = &edits[0];

        // Line-based approach replaces the entire line
        pretty_assertions::assert_eq!(edit.new_text, "hello universe");

        let expected_range: tower_lsp::lsp_types::Range = Range::new(
            Position::new(0, 0),  // Start of line 0
            Position::new(0, 11), // End of last character in line 0
        )
        .into_lsp(&line_index);

        pretty_assertions::assert_eq!(edit.range, expected_range);
    }

    #[test]
    fn test_compute_text_edits_multiline() {
        let old_text = "line1\nline2\nline3";
        let new_text = "line1\nmodified line2\nline3";
        let line_index = LineIndex::new(old_text, WideEncoding::Utf16);
        let edits = compute_text_edits(
            old_text,
            new_text,
            &FormatDefinitions::default(),
            &line_index,
        );

        pretty_assertions::assert_eq!(edits.len(), 1);

        let edit = &edits[0];

        // We're replacing the entire "line2\n" with "modified line2\n"
        pretty_assertions::assert_eq!(edit.new_text, "modified line2\n");

        let expected_range: tower_lsp::lsp_types::Range =
            Range::from(((1, 0), (2, 0))).into_lsp(&line_index);

        pretty_assertions::assert_eq!(edit.range, expected_range);
    }
}
