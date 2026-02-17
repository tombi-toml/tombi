#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    #[default]
    Lf,
    Crlf,
}

impl From<LineEnding> for &'static str {
    fn from(line_ending: LineEnding) -> Self {
        match line_ending {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}
