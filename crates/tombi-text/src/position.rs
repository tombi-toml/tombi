use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Sub},
};

use crate::{Column, Line, RelativePosition};

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize))]
pub struct Position {
    pub line: Line,
    pub column: Column,
}

impl Position {
    pub const MAX: Position = Position {
        line: Line::MAX,
        column: Column::MAX,
    };
    pub const MIN: Position = Position {
        line: Line::MIN,
        column: Column::MIN,
    };

    #[inline]
    pub const fn new(line: Line, column: Column) -> Self {
        Self { line, column }
    }

    #[inline]
    pub fn add_text(&self, text: &str) -> Self {
        (*self) + RelativePosition::of(text)
    }

    #[inline]
    pub fn char_at_left(&self, text: &str) -> Option<char> {
        text.split('\n')
            .nth(self.line as usize)
            .and_then(|line| line.chars().nth(self.column.saturating_sub(1) as usize))
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.line
            .cmp(&other.line)
            .then_with(|| self.column.cmp(&other.column))
    }
}

impl PartialOrd for Position {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<(Line, Column)> for Position {
    #[inline]
    fn from((line, column): (Line, Column)) -> Self {
        Self::new(line, column)
    }
}

impl Add<RelativePosition> for Position {
    type Output = Position;

    #[inline]
    fn add(self, rhs: RelativePosition) -> Self::Output {
        let line = self.line + rhs.line;
        let column = if rhs.line == 0 {
            self.column + rhs.column
        } else {
            rhs.column
        };
        Position::new(line, column)
    }
}

impl AddAssign<RelativePosition> for Position {
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

impl Sub<Position> for Position {
    type Output = RelativePosition;

    #[inline]
    fn sub(self, rhs: Position) -> Self::Output {
        assert!(rhs <= self);
        let line = self.line - rhs.line;
        let column = if line == 0 {
            self.column - rhs.column
        } else {
            self.column
        };
        RelativePosition { line, column }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_position_cmp() {
        use super::Position;

        let p1 = Position::new(1, 2);
        let p2 = Position::new(1, 3);
        let p3 = Position::new(2, 0);

        assert!(p1 < p2);
        assert!(p2 < p3);
        assert!(p1 < p3);
    }
}
