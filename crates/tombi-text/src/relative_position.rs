use std::{
    cmp::Ordering,
    ops::{Add, AddAssign},
};

use unicode_segmentation::UnicodeSegmentation;

use crate::{Column, Line};

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RelativePosition {
    pub line: Line,
    pub column: Column,
}

impl RelativePosition {
    pub fn of(text: &str) -> Self {
        let mut line = 0;
        let mut column = 0;
        for c in UnicodeSegmentation::graphemes(text, true) {
            if matches!(c, "\n" | "\r\n") {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        Self { line, column }
    }
}

impl Ord for RelativePosition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.line
            .cmp(&other.line)
            .then_with(|| self.column.cmp(&other.column))
    }
}

impl PartialOrd for RelativePosition {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<(Line, Column)> for RelativePosition {
    #[inline]
    fn from((line, column): (Line, Column)) -> Self {
        Self { line, column }
    }
}

impl From<char> for RelativePosition {
    #[inline]
    fn from(c: char) -> Self {
        if c == '\n' {
            Self { line: 1, column: 0 }
        } else {
            Self { line: 0, column: 1 }
        }
    }
}

impl From<crate::Position> for RelativePosition {
    #[inline]
    fn from(position: crate::Position) -> Self {
        Self {
            line: position.line,
            column: position.column,
        }
    }
}

impl Add for RelativePosition {
    type Output = RelativePosition;

    #[inline]
    fn add(self, rhs: RelativePosition) -> Self::Output {
        Self {
            line: self.line + rhs.line,
            column: if rhs.line == 0 {
                self.column + rhs.column
            } else {
                rhs.column
            },
        }
    }
}

impl AddAssign for RelativePosition {
    #[inline]
    fn add_assign(&mut self, rhs: RelativePosition) {
        self.line += rhs.line;
        if rhs.line == 0 {
            self.column += rhs.column;
        } else {
            self.column = rhs.column;
        }
    }
}

#[cfg(test)]
mod test {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("", (0, 0))]
    #[case("a", (0, 1))]
    #[case("abc\ndef\nghi", (2, 3))]
    #[case("abc\r\ndef\r\nghi", (2, 3))]
    #[case("🦅", (0, 1))]
    #[case("こんにちは", (0, 5))]
    #[case("Hello🦅World", (0, 11))]
    #[case("こんにちは🦅世界", (0, 8))]
    #[case("🦅\nこんにちは", (1, 5))]
    fn test_position(#[case] source: &str, #[case] expected: (Line, Column)) {
        pretty_assertions::assert_eq!(RelativePosition::of(source), expected.into());
    }

    #[test]
    fn test_add() {
        let pos1 = RelativePosition::from((0, 5));
        let pos2 = RelativePosition::from((0, 3));
        let result = pos1 + pos2;
        assert_eq!(result, RelativePosition::from((0, 8)));

        let pos1 = RelativePosition::from((1, 5));
        let pos2 = RelativePosition::from((0, 3));
        let result = pos1 + pos2;
        assert_eq!(result, RelativePosition::from((1, 8)));

        let pos1 = RelativePosition::from((1, 5));
        let pos2 = RelativePosition::from((1, 3));
        let result = pos1 + pos2;
        assert_eq!(result, RelativePosition::from((2, 3)));
    }
}
