#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum Error {
    #[error("expected key")]
    ExpectedKey,

    #[error("expected value")]
    ExpectedValue,

    #[error("unknown token")]
    UnknownToken,

    #[error("unknown line")]
    UnknownLine,

    #[error("expected '='")]
    ExpectedEqual,

    #[error("expected ']'")]
    ExpectedBracketEnd,

    #[error("expected ']]'")]
    ExpectedDoubleBracketEnd,

    #[error("expected '}}'")]
    ExpectedBraceEnd,

    #[error("expected '\\n' or comment")]
    ExpectedLineBreakOrComment,

    #[error("Invalid key")]
    InvalidKey,
}

impl From<Error> for String {
    fn from(val: Error) -> Self {
        val.to_string()
    }
}
