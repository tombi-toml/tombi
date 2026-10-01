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

    let Some(document_source) = backend.document_source(&text_document_uri) else {
        return Ok(None);
    };

    let mut cursor = document_source
        .line_index()
        .cursor(document_source.encoding_kind());
    let folding_ranges = create_folding_spans(&document_source.ast())
        .into_iter()
        .map(|FoldingSpan { span, kind }| {
            let range = cursor.lsp_range(span);
            FoldingRange {
                start_line: range.start.line,
                start_character: Some(range.start.character),
                end_line: range.end.line,
                end_character: Some(range.end.character),
                kind: Some(kind),
                collapsed_text: None,
            }
        })
        .collect_vec();

    if !folding_ranges.is_empty() {
        Ok(Some(folding_ranges))
    } else {
        Ok(None)
    }
}

/// A folding span whose position is converted at the end, in document order.
struct FoldingSpan {
    span: tombi_text::Span,
    kind: FoldingRangeKind,
}

fn create_folding_spans(root: &tombi_ast_syntax::Root<'_>) -> Vec<FoldingSpan> {
    let mut spans: Vec<FoldingSpan> = vec![];

    for node in root.nodes() {
        if let tombi_ast_syntax::TomlNode::KeyValue(key_value) = node {
            for folding_span in [key_value
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::Table(table) = node {
            for folding_span in itertools::chain!(
                table
                    .header_leading_comments()
                    .collect_vec()
                    .get_comment_folding_span(),
                table.get_region_folding_span(),
                table
                    .dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_span(),
            ) {
                spans.push(folding_span);
            }

            spans.extend(
                table
                    .key_value_groups()
                    .filter_map(DanglingCommentGroupOr::into_dangling_comment_group)
                    .flat_map(|comment_group| {
                        comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_span()
                    }),
            );
        } else if let tombi_ast_syntax::TomlNode::ArrayOfTable(array_of_table) = node {
            for folding_span in itertools::chain!(
                array_of_table
                    .header_leading_comments()
                    .collect_vec()
                    .get_comment_folding_span(),
                array_of_table.get_region_folding_span(),
                array_of_table
                    .dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_span(),
            ) {
                spans.push(folding_span);
            }

            spans.extend(
                array_of_table
                    .key_value_groups()
                    .filter_map(DanglingCommentGroupOr::into_dangling_comment_group)
                    .flat_map(|comment_group| {
                        comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_span()
                    }),
            );
        } else if let tombi_ast_syntax::TomlNode::Boolean(boolean) = node {
            for folding_span in [boolean
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::IntegerBin(integer_bin) = node {
            for folding_span in [integer_bin
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::IntegerOct(integer_oct) = node {
            for folding_span in [integer_oct
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::IntegerDec(integer_dec) = node {
            for folding_span in [integer_dec
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::IntegerHex(integer_hex) = node {
            for folding_span in [integer_hex
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::Float(float) = node {
            for folding_span in [float
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::BasicString(basic_string) = node {
            for folding_span in [basic_string
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::LiteralString(literal_string) = node {
            for folding_span in [literal_string
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::MultiLineBasicString(multi_line_basic_string) =
            node
        {
            for folding_span in [
                multi_line_basic_string
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_span(),
                multi_line_basic_string.get_region_folding_span(),
            ]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::MultiLineLiteralString(
            multi_line_literal_string,
        ) = node
        {
            for folding_span in [
                multi_line_literal_string
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_span(),
                multi_line_literal_string.get_region_folding_span(),
            ]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::OffsetDateTime(offset_date_time) = node {
            for folding_span in [offset_date_time
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::LocalDateTime(local_date_time) = node {
            for folding_span in [local_date_time
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::LocalDate(local_date) = node {
            for folding_span in [local_date
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::LocalTime(local_time) = node {
            for folding_span in [local_time
                .leading_comments()
                .collect_vec()
                .get_comment_folding_span()]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }
        } else if let tombi_ast_syntax::TomlNode::Array(array) = node {
            for folding_span in itertools::chain!(
                array
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_span(),
                array
                    .dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_span(),
                array.get_region_folding_span(),
            ) {
                spans.push(folding_span);
            }

            for group in array.value_with_comma_groups() {
                match group {
                    DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                        if let Some(folding_span) = comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_span()
                        {
                            spans.push(folding_span);
                        }
                    }
                    DanglingCommentGroupOr::ItemGroup(value_group) => {
                        for (_, comma) in value_group.value_or_key_values_with_comma() {
                            let Some(comma) = comma else {
                                continue;
                            };

                            if let Some(folding_span) = comma
                                .leading_comments()
                                .collect_vec()
                                .get_comment_folding_span()
                            {
                                spans.push(folding_span);
                            }
                        }
                    }
                }
            }
        } else if let tombi_ast_syntax::TomlNode::InlineTable(inline_table) = node {
            for folding_span in [
                inline_table
                    .leading_comments()
                    .collect_vec()
                    .get_comment_folding_span(),
                inline_table
                    .dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_span(),
                inline_table.get_region_folding_span(),
            ]
            .into_iter()
            .flatten()
            {
                spans.push(folding_span);
            }

            for group in inline_table.key_value_with_comma_groups() {
                match group {
                    DanglingCommentGroupOr::DanglingCommentGroup(comment_group) => {
                        if let Some(folding_span) = comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_span()
                        {
                            spans.push(folding_span);
                        }
                    }
                    DanglingCommentGroupOr::ItemGroup(key_value_group) => {
                        for (_, comma) in key_value_group.key_values_with_comma() {
                            let Some(comma) = comma else {
                                continue;
                            };

                            if let Some(folding_span) = comma
                                .leading_comments()
                                .collect_vec()
                                .get_comment_folding_span()
                            {
                                spans.push(folding_span);
                            }
                        }
                    }
                }
            }
        } else if let tombi_ast_syntax::TomlNode::Root(root) = node {
            for folding_span in itertools::chain!(
                root.dangling_comment_groups()
                    .map(|comment_group| comment_group.into_comments().collect_vec())
                    .collect_vec()
                    .get_comment_folding_span()
            ) {
                spans.push(folding_span);
            }

            spans.extend(
                root.key_value_groups()
                    .filter_map(DanglingCommentGroupOr::into_dangling_comment_group)
                    .flat_map(|comment_group| {
                        comment_group
                            .into_comments()
                            .collect_vec()
                            .get_comment_folding_span()
                    }),
            );
        }
    }

    spans
}

trait GetRegionFoldingSpan {
    fn get_folding_span(&self) -> Option<tombi_text::Span>;

    #[inline]
    fn get_region_folding_span(&self) -> Option<FoldingSpan> {
        self.get_folding_span().map(|span| FoldingSpan {
            span,
            kind: FoldingRangeKind::Region,
        })
    }
}

trait GetCommentFoldingSpan {
    fn get_folding_span(&self) -> Option<tombi_text::Span>;

    #[inline]
    fn get_comment_folding_span(&self) -> Option<FoldingSpan> {
        self.get_folding_span().map(|span| FoldingSpan {
            span,
            kind: FoldingRangeKind::Comment,
        })
    }
}

impl GetRegionFoldingSpan for tombi_ast_syntax::Table<'_> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        self.content_span().map(|span| {
            tombi_text::Span::new(
                span.start,
                self.last_sub_table()
                    .and_then(|t| t.get_folding_span())
                    .unwrap_or(span)
                    .end,
            )
        })
    }
}

impl GetRegionFoldingSpan for tombi_ast_syntax::ArrayOfTable<'_> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        self.content_span().map(|span| {
            tombi_text::Span::new(
                span.start,
                self.last_sub_table()
                    .and_then(|t| t.get_folding_span())
                    .unwrap_or(span)
                    .end,
            )
        })
    }
}

impl GetRegionFoldingSpan for tombi_ast_syntax::TableOrArrayOfTable<'_> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        match self {
            Self::Table(table) => table.get_folding_span(),
            Self::ArrayOfTable(array_of_table) => array_of_table.get_folding_span(),
        }
    }
}

impl GetRegionFoldingSpan for tombi_ast_syntax::Array<'_> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        let start_position = self.bracket_start()?.span().start;
        let end_position = self.bracket_end()?.span().end;

        Some(tombi_text::Span::new(start_position, end_position))
    }
}

impl GetRegionFoldingSpan for tombi_ast_syntax::InlineTable<'_> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        let start_position = self.brace_start()?.span().start;
        let end_position = self.brace_end()?.span().end;

        Some(tombi_text::Span::new(start_position, end_position))
    }
}

impl GetRegionFoldingSpan for tombi_ast_syntax::MultiLineBasicString<'_> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        let token = self.token()?;

        token.text().contains('\n').then(|| token.span())
    }
}

impl GetRegionFoldingSpan for tombi_ast_syntax::MultiLineLiteralString<'_> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        let token = self.token()?;

        token.text().contains('\n').then(|| token.span())
    }
}

impl GetCommentFoldingSpan for Vec<tombi_ast_syntax::LeadingComment<'_>> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        let first = self.first()?;
        let last = self.last()?;
        Some(tombi_text::Span::new(
            first.syntax().span().start,
            last.syntax().span().end,
        ))
    }
}

impl GetCommentFoldingSpan for Vec<tombi_ast_syntax::DanglingComment<'_>> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        let first = self.first()?;
        let last = self.last()?;
        Some(tombi_text::Span::new(
            first.syntax().span().start,
            last.syntax().span().end,
        ))
    }
}

impl GetCommentFoldingSpan for Vec<Vec<tombi_ast_syntax::DanglingComment<'_>>> {
    fn get_folding_span(&self) -> Option<tombi_text::Span> {
        let first = self.iter().find(|group| !group.is_empty())?.iter().next()?;
        let last = self
            .iter()
            .rev()
            .find(|group| !group.is_empty())?
            .iter()
            .next_back()?;

        // A comment ends at the end of its line, so the comments are on one line only if they are the same.
        if first.syntax().span() == last.syntax().span() {
            return None;
        }

        Some(tombi_text::Span::new(
            first.syntax().span().start,
            last.syntax().span().end,
        ))
    }
}

#[cfg(test)]
mod tests {
    use tombi_ast_syntax::{AstNode, TableOrArrayOfTable};

    fn last_sub_tables(source: &str) -> Vec<(String, Option<String>)> {
        let parsed = tombi_parser::parse(source);
        let root = parsed.try_root().ok().unwrap();
        let header =
            |item: &TableOrArrayOfTable<'_>| item.header().unwrap().syntax().text().to_string();
        root.table_or_array_of_tables()
            .map(|item| {
                let last = match &item {
                    TableOrArrayOfTable::Table(table) => table.last_sub_table(),
                    TableOrArrayOfTable::ArrayOfTable(array_of_table) => {
                        array_of_table.last_sub_table()
                    }
                };
                (header(&item), last.as_ref().map(header))
            })
            .collect()
    }

    #[test]
    fn last_sub_table_with_unconvertible_trailing_key() {
        let actual = last_sub_tables("[a]\n[a.\"\\q\"]\n[other]\n");
        assert_eq!(actual[0].1.as_deref().map(str::trim), Some("a.\"\\q\""));
        assert_eq!(actual[1].1, None);
    }

    #[test]
    fn last_sub_table_of_nested_and_sibling_headers() {
        let source = "[a]\n[a.b]\n[a.b.c]\n[a.d]\n[other]\n[[x]]\n[x.y]\n[[x]]\n[[x.z]]\n[[x.z]]\n";
        let expected = [
            ("a", Some("a.d")),
            ("a.b", Some("a.b.c")),
            ("a.b.c", None),
            ("a.d", None),
            ("other", None),
            ("x", Some("x.y")),
            ("x.y", None),
            ("x", Some("x.z")),
            ("x.z", None),
            ("x.z", None),
        ];
        let actual = last_sub_tables(source);
        assert_eq!(actual.len(), expected.len());
        for ((header, last), (expected_header, expected_last)) in actual.iter().zip(expected) {
            assert_eq!(header.trim(), expected_header);
            assert_eq!(last.as_deref().map(str::trim), expected_last, "{header}");
        }
    }
}
