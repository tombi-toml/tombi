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
    /// Resolves separators for an insertion between the values of a TOML array.
    pub fn new_array_item(
        values: impl IntoIterator<Item = tombi_text::Span>,
        text: &str,
        offset: tombi_text::Offset,
        hint: Option<Self>,
    ) -> Self {
        if let Some(hint @ Self::InArray { .. }) = hint {
            return hint;
        }
        let mut previous = None;
        let mut next = None;
        for span in values {
            if span.end <= offset {
                previous = Some(span);
            } else if span.start >= offset && next.is_none() {
                next = Some(span);
            }
        }
        let has_leading_comma = matches!(
            hint,
            Some(Self::Comma {
                leading_comma: Some(_),
                ..
            })
        );
        let has_trailing_comma = matches!(
            hint,
            Some(Self::Comma {
                trailing_comma: Some(_),
                ..
            })
        );
        let add_leading_comma = previous.filter(|_| !has_leading_comma).map(|previous| {
            let mut start_offset = previous.end;
            let mut whitespace = &text[tombi_text::Span::new(start_offset, offset)];
            if !whitespace.chars().all(char::is_whitespace) {
                start_offset = offset;
                whitespace = "";
            }
            let line_breaks = whitespace.chars().filter(|ch| *ch == '\n').count() as u32;
            let indent = whitespace
                .rsplit('\n')
                .next()
                .unwrap_or_default()
                .chars()
                .count() as u32;
            AddLeadingComma {
                start_offset,
                line_breaks,
                indent,
            }
        });
        Self::InArray {
            add_leading_comma,
            add_trailing_comma: (next.is_some() && !has_trailing_comma).then_some(AddTrailingComma),
        }
    }
}
