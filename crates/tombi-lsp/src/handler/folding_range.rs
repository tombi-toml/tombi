use itertools::Itertools;
use tombi_ast_syntax::{AstNode, DanglingCommentGroupOr};
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind, FoldingRangeParams};

use crate::backend::Backend;

pub async fn handle_folding_range(
    backend: &Backend,
    params: FoldingRangeParams,
) -> Result<Option<Vec<FoldingRange>>, tower_lsp::jsonrpc::Error> {
    log::info!("handle_folding_range");
    log::trace!("{:?}", params);

    let FoldingRangeParams { text_document, .. } = params;
    let text_document_uri = text_document.uri.into();

    let Ok(document_sources) = backend.document_sources.try_read() else {
        return Ok(None);
    };
    let Some(document_source) = document_sources.get(&text_document_uri) else {
        return Ok(None);
    };

    let folding_ranges = create_folding_ranges(&document_source.ast());

    if !folding_ranges.is_empty() {
        Ok(Some(folding_ranges))
    } else {
        Ok(None)
    }
}

fn create_folding_ranges(root: &tombi_ast_syntax::Root) -> Vec<FoldingRange> {
    let mut ranges: Vec<FoldingRange> = vec![];

    for node in root.nodes() {
        if let tombi_ast_syntax::TomlNode::KeyValue(key_value) = node {
            for folding_range in [key_value
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::Table(table) = node {
            for folding_range in itertools::chain!(
                table
                    .header_leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                table.get_region_folding_range(),
                table
                    .dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_range(),
            ) {
                ranges.push(folding_range);
            }

            ranges.extend(
                table
                    .key_value_groups()
                    .filter_map(DanglingCommentGroupOr::into_dangling_comment_group)
                    .flat_map(|comment_group| {
                        comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_range()
                    }),
            );
        } else if let tombi_ast_syntax::TomlNode::ArrayOfTable(array_of_table) = node {
            for folding_range in itertools::chain!(
                array_of_table
                    .header_leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                array_of_table.get_region_folding_range(),
                array_of_table
                    .dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_range(),
            ) {
                ranges.push(folding_range);
            }

            ranges.extend(
                array_of_table
                    .key_value_groups()
                    .filter_map(DanglingCommentGroupOr::into_dangling_comment_group)
                    .flat_map(|comment_group| {
                        comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_range()
                    }),
            );
        } else if let tombi_ast_syntax::TomlNode::Boolean(boolean) = node {
            for folding_range in [boolean
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::IntegerBin(integer_bin) = node {
            for folding_range in [integer_bin
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::IntegerOct(integer_oct) = node {
            for folding_range in [integer_oct
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::IntegerDec(integer_dec) = node {
            for folding_range in [integer_dec
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::IntegerHex(integer_hex) = node {
            for folding_range in [integer_hex
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::Float(float) = node {
            for folding_range in [float
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::BasicString(basic_string) = node {
            for folding_range in [basic_string
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::LiteralString(literal_string) = node {
            for folding_range in [literal_string
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::MultiLineBasicString(multi_line_basic_string) =
            node
        {
            for folding_range in [
                multi_line_basic_string
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                multi_line_basic_string.get_region_folding_range(),
            ]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::MultiLineLiteralString(
            multi_line_literal_string,
        ) = node
        {
            for folding_range in [
                multi_line_literal_string
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                multi_line_literal_string.get_region_folding_range(),
            ]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::OffsetDateTime(offset_date_time) = node {
            for folding_range in [offset_date_time
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::LocalDateTime(local_date_time) = node {
            for folding_range in [local_date_time
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::LocalDate(local_date) = node {
            for folding_range in [local_date
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::LocalTime(local_time) = node {
            for folding_range in [local_time
                .leading_comments()
                .collect_vec()
                .get_comment_folding_range()]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }
        } else if let tombi_ast_syntax::TomlNode::Array(array) = node {
            for folding_range in itertools::chain!(
                array
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                array
                    .dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_range(),
                array.get_region_folding_range(),
            ) {
                ranges.push(folding_range);
            }

            for group in array.value_with_comma_groups() {
                match group {
                    DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                        if let Some(folding_range) = comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_range()
                        {
                            ranges.push(folding_range);
                        }
                    }
                    DanglingCommentGroupOr::ItemGroup(value_group) => {
                        for (_, comma) in value_group.value_or_key_values_with_comma() {
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
                    }
                }
            }
        } else if let tombi_ast_syntax::TomlNode::InlineTable(inline_table) = node {
            for folding_range in [
                inline_table
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_range(),
                inline_table
                    .dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_range(),
                inline_table.get_region_folding_range(),
            ]
            .into_iter()
            .flatten()
            {
                ranges.push(folding_range);
            }

            for group in inline_table.key_value_with_comma_groups() {
                match group {
                    DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                        if let Some(folding_range) = comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_range()
                        {
                            ranges.push(folding_range);
                        }
                    }
                    DanglingCommentGroupOr::ItemGroup(key_value_group) => {
                        for (_, comma) in key_value_group.key_values_with_comma() {
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
                    }
                }
            }
        } else if let tombi_ast_syntax::TomlNode::Root(root) = node {
            for folding_range in itertools::chain!(
                root.dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_range()
            ) {
                ranges.push(folding_range);
            }

            ranges.extend(
                root.key_value_groups()
                    .filter_map(DanglingCommentGroupOr::into_dangling_comment_group)
                    .flat_map(|comment_group| {
                        comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_range()
                    }),
            );
        }
    }

    ranges
}

trait GetRegionFoldingRange {
    fn get_folding_range(&self) -> Option<tombi_text::Range>;

    #[inline]
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
}

trait GetCommentFoldingRange {
    fn get_folding_range(&self) -> Option<tombi_text::Range>;

    #[inline]
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

impl GetRegionFoldingRange for tombi_ast_syntax::Table {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        self.content_range().map(|range| {
            tombi_text::Range::new(
                range.start,
                self.sub_tables()
                    .last()
                    .and_then(|t| t.get_folding_range())
                    .unwrap_or(range)
                    .end,
            )
        })
    }
}

impl GetRegionFoldingRange for tombi_ast_syntax::ArrayOfTable {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        self.content_range().map(|range| {
            tombi_text::Range::new(
                range.start,
                self.sub_tables()
                    .last()
                    .and_then(|t| t.get_folding_range())
                    .unwrap_or(range)
                    .end,
            )
        })
    }
}

impl GetRegionFoldingRange for tombi_ast_syntax::TableOrArrayOfTable {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        match self {
            Self::Table(table) => table.get_folding_range(),
            Self::ArrayOfTable(array_of_table) => array_of_table.get_folding_range(),
        }
    }
}

impl GetRegionFoldingRange for tombi_ast_syntax::Array {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let start_position = self.bracket_start()?.range().start;
        let end_position = self.bracket_end()?.range().end;

        Some(tombi_text::Range::new(start_position, end_position))
    }
}

impl GetRegionFoldingRange for tombi_ast_syntax::InlineTable {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let start_position = self.brace_start()?.range().start;
        let end_position = self.brace_end()?.range().end;

        Some(tombi_text::Range::new(start_position, end_position))
    }
}

impl GetRegionFoldingRange for tombi_ast_syntax::MultiLineBasicString {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let token = self.token()?;
        let range = token.range();

        if range.start.line != range.end.line {
            Some(range)
        } else {
            None
        }
    }
}

impl GetRegionFoldingRange for tombi_ast_syntax::MultiLineLiteralString {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let token = self.token()?;
        let range = token.range();

        if range.start.line != range.end.line {
            Some(range)
        } else {
            None
        }
    }
}

impl GetCommentFoldingRange for Vec<tombi_ast_syntax::LeadingComment> {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let first = self.first()?;
        let last = self.last()?;
        Some(tombi_text::Range::new(
            first.syntax().range().start,
            last.syntax().range().end,
        ))
    }
}

impl GetCommentFoldingRange for Vec<tombi_ast_syntax::DanglingComment> {
    fn get_folding_range(&self) -> Option<tombi_text::Range> {
        let first = self.first()?;
        let last = self.last()?;
        Some(tombi_text::Range::new(
            first.syntax().range().start,
            last.syntax().range().end,
        ))
    }
}

impl GetCommentFoldingRange for Vec<Vec<tombi_ast_syntax::DanglingComment>> {
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
