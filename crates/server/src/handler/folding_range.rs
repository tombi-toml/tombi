use ast::AstNode;
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_folding_range(
    backend: &Backend,
    FoldingRangeParams { text_document, .. }: FoldingRangeParams,
) -> Result<Option<Vec<FoldingRange>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_folding_range");

    let toml_version = backend.toml_version().await.unwrap_or_default();

    let Some(Ok(root)) = backend.try_get_ast(&text_document.uri, toml_version).await else {
        return Ok(None);
    };

    let folding_ranges = create_folding_ranges(root);

    if !folding_ranges.is_empty() {
        Ok(Some(folding_ranges))
    } else {
        Ok(None)
    }
}

fn create_folding_ranges(root: ast::Root) -> Vec<FoldingRange> {
    let mut ranges: Vec<FoldingRange> = vec![];

    for node in root.syntax().descendants() {
        if let Some(table) = ast::Table::cast(node.to_owned()) {
            let start_position = table.header().unwrap().range().start();
            let end_position = table
                .subtables()
                .last()
                .map_or(table.range().end(), |t| t.range().end());

            ranges.push(FoldingRange {
                start_line: start_position.line(),
                start_character: None,
                end_line: end_position.line(),
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        } else if let Some(array_of_tables) = ast::ArrayOfTables::cast(node.to_owned()) {
            let start_position = array_of_tables.header().unwrap().range().start();
            let end_position = array_of_tables.range().end();

            ranges.push(FoldingRange {
                start_line: start_position.line(),
                start_character: None,
                end_line: end_position.line(),
                end_character: None,
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        } else if let Some(array) = ast::Array::cast(node.to_owned()) {
            let start_position = array.bracket_start().unwrap().range().start();
            let end_position = array.bracket_end().unwrap().range().end();

            ranges.push(FoldingRange {
                start_line: start_position.line(),
                start_character: Some(start_position.column()),
                end_line: end_position.line(),
                end_character: Some(end_position.column()),
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        } else if let Some(inline_table) = ast::InlineTable::cast(node.to_owned()) {
            let start_position = inline_table.brace_start().unwrap().range().start();
            let end_position = inline_table.brace_end().unwrap().range().end();

            ranges.push(FoldingRange {
                start_line: start_position.line(),
                start_character: Some(start_position.column()),
                end_line: end_position.line(),
                end_character: Some(end_position.column()),
                kind: Some(FoldingRangeKind::Region),
                collapsed_text: None,
            });
        }
    }

    ranges
}
