use itertools::Itertools;
use tombi_ast::AstNode;
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

use crate::backend::Backend;

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_folding_range(
    backend: &Backend,
    params: FoldingRangeParams,
) -> Result<Option<Vec<FoldingRange>>, tower_lsp::jsonrpc::Error> {
    tracing::info!("handle_folding_range");
    tracing::trace!(?params);

    let FoldingRangeParams { text_document, .. } = params;
    let text_document_uri = text_document.uri.into();

    let Some((root, _)) = backend.get_ast_and_diagnostics(&text_document_uri).await else {
        return Ok(None);
    };

    let folding_ranges = create_folding_ranges(root);

    if !folding_ranges.is_empty() {
        Ok(Some(folding_ranges))
    } else {
        Ok(None)
    }
}

fn create_folding_ranges(root: tombi_ast::Root) -> Vec<FoldingRange> {
    let mut ranges: Vec<FoldingRange> = vec![];

    for node in root.syntax().descendants() {
        if let Some(key_value) = tombi_ast::KeyValue::cast(node.to_owned()) {
            for folding_range in [key_value
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(table) = tombi_ast::Table::cast(node.to_owned()) {
            for folding_range in [
                table
                    .header_leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                table.get_region_folding_range(),
                table
                    .key_values_begin_dangling_comments()
                    .get_comment_folding_range(),
                table
                    .key_values_end_dangling_comments()
                    .get_comment_folding_range(),
            ]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(array_of_table) = tombi_ast::ArrayOfTable::cast(node.to_owned()) {
            for folding_range in [
                array_of_table
                    .header_leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                array_of_table.get_region_folding_range(),
                array_of_table
                    .key_values_begin_dangling_comments()
                    .get_comment_folding_range(),
                array_of_table
                    .key_values_end_dangling_comments()
                    .get_comment_folding_range(),
            ]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(boolean) = tombi_ast::Boolean::cast(node.to_owned()) {
            for folding_range in [boolean
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(integer_bin) = tombi_ast::IntegerBin::cast(node.to_owned()) {
            for folding_range in [integer_bin
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(integer_oct) = tombi_ast::IntegerOct::cast(node.to_owned()) {
            for folding_range in [integer_oct
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(integer_dec) = tombi_ast::IntegerDec::cast(node.to_owned()) {
            for folding_range in [integer_dec
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(integer_hex) = tombi_ast::IntegerHex::cast(node.to_owned()) {
            for folding_range in [integer_hex
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(float) = tombi_ast::Float::cast(node.to_owned()) {
            for folding_range in [float
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(basic_string) = tombi_ast::BasicString::cast(node.to_owned()) {
            for folding_range in [basic_string
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(literal_string) = tombi_ast::LiteralString::cast(node.to_owned()) {
            for folding_range in [literal_string
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(multi_line_basic_string) =
            tombi_ast::MultiLineBasicString::cast(node.to_owned())
        {
            for folding_range in [multi_line_basic_string
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(multi_line_literal_string) =
            tombi_ast::MultiLineLiteralString::cast(node.to_owned())
        {
            for folding_range in [multi_line_literal_string
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(offset_date_time) = tombi_ast::OffsetDateTime::cast(node.to_owned()) {
            for folding_range in [offset_date_time
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(local_date_time) = tombi_ast::LocalDateTime::cast(node.to_owned()) {
            for folding_range in [local_date_time
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(local_date) = tombi_ast::LocalDate::cast(node.to_owned()) {
            for folding_range in [local_date
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(local_time) = tombi_ast::LocalTime::cast(node.to_owned()) {
            for folding_range in [local_time
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let Some(array) = tombi_ast::Array::cast(node.to_owned()) {
            for folding_range in [
                array
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                array
                    .inner_begin_dangling_comments()
                    .get_comment_folding_range(),
                array.get_region_folding_range(),
                array
                    .inner_end_dangling_comments()
                    .get_comment_folding_range(),
            ]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
            for (_, comma) in array.values_with_comma() {
                let Some(comma) = comma else {
                    continue;
                };

                if let Some(folding_range) = comma
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_range()
                {
                    ranges.push(folding_range);
                }
            }
        } else if let Some(inline_table) = tombi_ast::InlineTable::cast(node.to_owned()) {
            for folding_range in [
                inline_table
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                inline_table
                    .inner_begin_dangling_comments()
                    .get_comment_folding_range(),
                inline_table.get_region_folding_range(),
                inline_table
                    .inner_end_dangling_comments()
                    .get_comment_folding_range(),
            ]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
            for (_, comma) in inline_table.key_values_with_comma() {
                let Some(comma) = comma else {
                    continue;
                };

                if let Some(folding_range) = comma
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_range()
                {
                    ranges.push(folding_range);
                }
            }
        } else if let Some(root) = tombi_ast::Root::cast(node.to_owned()) {
            for folding_range in [
                root.key_values_begin_dangling_comments()
                    .get_comment_folding_range(),
                root.key_values_end_dangling_comments()
                    .get_comment_folding_range(),
            ]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        }
    }

    ranges
}

trait GetFoldingRange {
    fn get_folding_range(&self) -> Option<tombi_text::Range>;

    fn get_region_folding_range(&self) -> Option<FoldingRange> {
        self.get_folding_range().map(|range| FoldingRange {
            start_line: range.start.line,
            start_character: Some(range.start.column),
            end_line: range.end.line,
            end_character: Some(range.end.column),
            kind: Some(FoldingRangeKind::Region),
            collapsed_text: None,
        })
    }

    fn get_comment_folding_range(&self) -> Option<FoldingRange> {
        self.get_folding_range().map(|range| FoldingRange {
            start_line: range.start.line,
            start_character: Some(range.start.column),
            end_line: range.end.line,
            end_character: Some(range.end.column),
            kind: Some(FoldingRangeKind::Comment),
            collapsed_text: None,
        })
    }
}

impl GetFoldingRange for tombi_ast::Table {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        use tombi_syntax::{SyntaxKind::*, T};

        let children_with_tokens = self.syntax().children_with_tokens().collect_vec();
        let first_child = children_with_tokens
            .iter()
            .find(|child| matches!(child.kind(), T!('[')));
        let last_child = children_with_tokens
            .iter()
            .rev()
            .find(|child| !matches!(child.kind(), WHITESPACE | LINE_BREAK));

        match (first_child, last_child) {
            (Some(first), Some(last)) => Some(tombi_text::Range::new(
                first.range().start,
                self.subtables()
                    .last()
                    .and_then(|t| t.get_folding_range())
                    .unwrap_or(last.range())
                    .end,
            )),
            _ => None,
        }
    }
}

impl GetFoldingRange for tombi_ast::ArrayOfTable {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        use tombi_syntax::{SyntaxKind::*, T};

        let children_with_tokens = self.syntax().children_with_tokens().collect_vec();
        let first_child = children_with_tokens
            .iter()
            .find(|child| matches!(child.kind(), T!("[[")));
        let last_child = children_with_tokens
            .iter()
            .rev()
            .find(|child| !matches!(child.kind(), WHITESPACE | LINE_BREAK));

        match (first_child, last_child) {
            (Some(first), Some(last)) => Some(tombi_text::Range::new(
                first.range().start,
                self.subtables()
                    .last()
                    .and_then(|t| t.get_folding_range())
                    .unwrap_or(last.range())
                    .end,
            )),
            _ => None,
        }
    }
}

impl GetFoldingRange for tombi_ast::TableOrArrayOfTable {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        match self {
            Self::Table(table) => table.get_folding_range(),
            Self::ArrayOfTable(array_of_table) => array_of_table.get_folding_range(),
        }
    }
}

impl GetFoldingRange for tombi_ast::Array {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let start_position = self.bracket_start()?.range().start;
        let end_position = self.bracket_end()?.range().end;

        Some(tombi_text::Range::new(start_position, end_position))
    }
}

impl GetFoldingRange for tombi_ast::InlineTable {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let start_position = self.brace_start()?.range().start;
        let end_position = self.brace_end()?.range().end;

        Some(tombi_text::Range::new(start_position, end_position))
    }
}

impl GetFoldingRange for Vec<tombi_ast::LeadingComment> {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let first = self.first()?;
        let last = self.last()?;
        Some(tombi_text::Range::new(
            first.syntax().range().start,
            last.syntax().range().end,
        ))
    }
}

impl GetFoldingRange for Vec<Vec<tombi_ast::BeginDanglingComment>> {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let first = self.iter().find(|group| !group.is_empty())?.iter().next()?;
        let last = self
            .iter()
            .rev()
            .find(|group| !group.is_empty())?
            .iter()
            .next_back()?;

        if first.syntax().range().start.line == last.syntax().range().end.line {
            return None;
        }

        Some(tombi_text::Range::new(
            first.syntax().range().start,
            last.syntax().range().end,
        ))
    }
}

impl GetFoldingRange for Vec<Vec<tombi_ast::EndDanglingComment>> {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let first = self.iter().find(|group| !group.is_empty())?.iter().next()?;
        let last = self
            .iter()
            .rev()
            .find(|group| !group.is_empty())?
            .iter()
            .next_back()?;

        if first.syntax().range().start.line == last.syntax().range().end.line {
            return None;
        }

        Some(tombi_text::Range::new(
            first.syntax().range().start,
            last.syntax().range().end,
        ))
    }
}
