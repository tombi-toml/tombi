#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionHint {
    InTableHeader {
        is_array_of_table: bool,
    },
    InArray {
        add_leading_comma: Option<AddLeadingComma>,
        add_trailing_comma: Option<AddTrailingComma>,
    },
    DotTrigger {
        span: tombi_text::Span,
        cleanup_span: tombi_text::Span,
    },
    EqualTrigger {
        span: tombi_text::Span,
        cleanup_span: tombi_text::Span,
    },
    Comma {
        leading_comma: Option<CommaHint>,
        trailing_comma: Option<CommaHint>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommaHint {
    pub span: tombi_text::Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddLeadingComma {
    pub start_offset: tombi_text::Offset,
    /// The number of line breaks between the start and the cursor, kept after the comma.
    pub line_breaks: u32,
    /// The column of the cursor, kept as the indent after the line breaks.
    pub indent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddTrailingComma;

impl CompletionHint {
    /// The hint for inserting a new item into the array at `offset`. `item_spans` are the spans of the array's items.
    /// The commas already written around `offset` are not added again.
    /// `line_index` keeps the line breaks and the indent before `offset` in the leading comma.
    pub fn new_in_array(
        completion_hint: Option<CompletionHint>,
        line_index: &tombi_text::LineIndex<'_>,
        offset: tombi_text::Offset,
        item_spans: impl IntoIterator<Item = tombi_text::Span>,
    ) -> Self {
        let mut previous_end = None;
        let mut has_next_item = false;
        for span in item_spans {
            if span.end <= offset {
                previous_end = Some(span.end);
            }
            has_next_item |= span.start >= offset;
        }

        let (has_leading_comma, has_trailing_comma) = match completion_hint {
            Some(CompletionHint::Comma {
                leading_comma,
                trailing_comma,
            }) => (leading_comma.is_some(), trailing_comma.is_some()),
            _ => (false, false),
        };

        Self::InArray {
            add_leading_comma: previous_end
                .filter(|_| !has_leading_comma)
                .map(|start_offset| AddLeadingComma {
                    start_offset,
                    line_breaks: line_index
                        .line(offset)
                        .saturating_sub(line_index.line(start_offset)),
                    indent: line_index
                        .position(offset, tombi_text::EncodingKind::GraphemeCluster)
                        .column,
                }),
            add_trailing_comma: (has_next_item && !has_trailing_comma).then_some(AddTrailingComma),
        }
    }
}
