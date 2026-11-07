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
    /// The end of the item before the new item. The comma is inserted here.
    pub start_offset: tombi_text::Offset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddTrailingComma;

impl CompletionHint {
    /// The hint for inserting a new item into the array at `offset`. `item_spans` are the spans of the array's items.
    /// The commas already written around `offset` are not added again.
    pub fn new_in_array(
        completion_hint: Option<CompletionHint>,
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
                .map(|start_offset| AddLeadingComma { start_offset }),
            add_trailing_comma: (has_next_item && !has_trailing_comma).then_some(AddTrailingComma),
        }
    }
}
