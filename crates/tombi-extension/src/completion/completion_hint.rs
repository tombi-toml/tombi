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
