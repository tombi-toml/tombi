#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionHint {
    InTableHeader,
    InArray,
    DotTrigger {
        range: tombi_text::Range,
    },
    EqualTrigger {
        range: tombi_text::Range,
    },
    LastComma {
        range: tombi_text::Range,
    },
    NeedHeadComma {
        start_position: tombi_text::Position,
    },
    NeedTailComma,
}
