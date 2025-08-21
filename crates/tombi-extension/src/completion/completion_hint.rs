#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionHint {
    InTableHeader,
    InArray {
        add_leading_comma: Option<AddLeadingComma>,
        add_trailing_comma: Option<AddTrailingComma>,
    },
    DotTrigger {
        range: tombi_text::Range,
    },
    EqualTrigger {
        range: tombi_text::Range,
    },
    LastComma {
        range: tombi_text::Range,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddLeadingComma {
    pub start_position: tombi_text::Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddTrailingComma;
