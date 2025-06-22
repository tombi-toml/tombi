use itertools::Either;
use tombi_config::FormatOptions;
use tombi_formatter::formatter::definitions::FormatDefinitions;
use tombi_text::{Position, Range};
use tower_lsp::lsp_types::{
    notification::PublishDiagnostics, DocumentFormattingParams, PublishDiagnosticsParams, TextEdit,
};

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_formatting(
    backend: &Backend,
    params: DocumentFormattingParams,
) -> Result<Option<Vec<TextEdit>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_formatting");
    tracing::trace!(?params);

    let DocumentFormattingParams { text_document, .. } = params;

    let config = backend.config().await;

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

    let Some(root) = backend.get_incomplete_ast(&text_document.uri).await else {
        return Ok(None);
    };

    let source_schema = backend
        .schema_store
        .resolve_source_schema_from_ast(&root, Some(Either::Left(&text_document.uri)))
        .await
        .ok()
        .flatten();

    let (toml_version, _) = backend.source_toml_version(source_schema.as_ref()).await;

    let mut document_sources = backend.document_sources.write().await;
    let Some(document_source) = document_sources.get_mut(&text_document.uri) else {
        return Ok(None);
    };

    let formatter_definitions = FormatDefinitions::default();

    match tombi_formatter::Formatter::new(
        toml_version,
        &formatter_definitions,
        backend
            .config()
            .await
            .format
            .as_ref()
            .unwrap_or(&FormatOptions::default()),
        Some(Either::Left(&text_document.uri)),
        &backend.schema_store,
    )
    .format(&document_source.text)
    .await
    {
        Ok(new_text) => {
            if new_text != document_source.text {
                let edits =
                    compute_text_edits(&document_source.text, &new_text, &formatter_definitions);
                document_source.text = new_text.clone();

                return Ok(Some(edits));
            } else {
                tracing::debug!("no change");
                backend
                    .client
                    .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                        uri: text_document.uri,
                        diagnostics: Vec::with_capacity(0),
                        version: Some(document_source.version),
                    })
                    .await;
            }
        }
        Err(diagnostics) => {
            tracing::error!("failed to format");
            backend
                .client
                .send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri: text_document.uri,
                    diagnostics: diagnostics.into_iter().map(Into::into).collect(),
                    version: Some(document_source.version),
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
) -> Vec<TextEdit> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let line_ending = formatter_definitions.line_ending.unwrap_or_default().into();

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
    let replacement_text = if replacement_lines.is_empty() {
        String::new()
    } else {
        // Reconstruct with line breaks, being careful about the last line
        let mut result = replacement_lines.join(line_ending);

        // If we're not replacing to the end of the text and the original text doesn't end with a newline,
        // we need to be careful about trailing newlines
        if old_end_line < old_lines.len() || old_text.ends_with(line_ending) {
            result.push_str(line_ending);
        }

        result
    };

    // Calculate positions
    let start_pos = Position::new(old_start_line as u32, 0);
    let end_pos = if old_end_line < old_lines.len() {
        Position::new(old_end_line as u32, 0)
    } else {
        // End of document - need to get the column position
        let last_line = old_lines.last().unwrap_or(&"");
        Position::new(
            (old_lines.len() - 1) as u32,
            last_line.chars().count() as u32,
        )
    };

    vec![TextEdit {
        range: Range::new(start_pos, end_pos).into(),
        new_text: replacement_text,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tombi_text::{Position, Range};

    #[test]
    fn test_compute_text_edits_no_changes() {
        let old_text = "hello world";
        let new_text = "hello world";
        let edits = compute_text_edits(old_text, new_text, &FormatDefinitions::default());
        assert!(edits.is_empty());
    }

    #[test]
    fn test_compute_text_edits_simple_replacement() {
        let old_text = "hello world";
        let new_text = "hello universe";
        let edits = compute_text_edits(old_text, new_text, &FormatDefinitions::default());

        assert_eq!(edits.len(), 1);
        let edit = &edits[0];
        // Line-based approach replaces the entire line
        assert_eq!(edit.new_text, "hello universe");

        let expected_range: tower_lsp::lsp_types::Range = Range::new(
            Position::new(0, 0),  // Start of line 0
            Position::new(0, 11), // End of last character in line 0
        )
        .into();
        assert_eq!(edit.range, expected_range);
    }

    #[test]
    fn test_compute_text_edits_multiline() {
        let old_text = "line1\nline2\nline3";
        let new_text = "line1\nmodified line2\nline3";
        let edits = compute_text_edits(old_text, new_text, &FormatDefinitions::default());

        assert_eq!(edits.len(), 1);
        let edit = &edits[0];

        // We're replacing the entire "line2\n" with "modified line2\n"
        assert_eq!(edit.new_text, "modified line2\n");

        let expected_range: tower_lsp::lsp_types::Range = Range::new(
            Position::new(1, 0), // Start of line 1 (line2)
            Position::new(2, 0), // Start of line 2 (line3)
        )
        .into();
        assert_eq!(edit.range, expected_range);
    }
}
