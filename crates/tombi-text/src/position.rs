use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Sub},
};

use crate::{Column, Line, PointerAlign, RelativePosition};

#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Position {
    pub line: Line,
    pub column: Column,

    _align: PointerAlign,
}

impl Position {
    pub const MAX: Position = Position::new(Line::MAX, Column::MAX);
    pub const MIN: Position = Position::new(Line::MIN, Column::MIN);

    #[inline]
    pub const fn new(line: Line, column: Column) -> Self {
        Self {
            line,
            column,
            _align: PointerAlign([]),
        }
    }

    #[inline]
    pub fn add_text(&self, text: &str) -> Self {
        (*self) + RelativePosition::of(text)
    }

    #[expect(clippy::inline_always)] // Because this is a no-op on 64-bit platforms.
    #[inline(always)]
    const fn as_u64(self) -> u64 {
        if cfg!(target_endian = "little") {
            ((self.column as u64) << 32) | (self.line as u64)
        } else {
            ((self.line as u64) << 32) | (self.column as u64)
        }
    }
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("line", &self.line)
            .field("column", &self.column)
            .finish()
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
        if rhs > self {
            tracing::warn!(
                "Invalid tombi_text::Position: rhs: {:?} > self: {:?}",
                rhs,
                self
            );
            return RelativePosition { line: 0, column: 0 };
        }
        let line = self.line - rhs.line;
        let column = if line == 0 {
            self.column - rhs.column
        } else {
            self.column
        };
        RelativePosition { line, column }
    }
}

impl PartialEq for Position {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if cfg!(target_pointer_width = "64") {
            self.as_u64() == other.as_u64()
        } else {
            self.line == other.line && self.column == other.column
        }
    }
}

impl Eq for Position {}

impl std::hash::Hash for Position {
    #[inline] // We exclusively use `FxHasher`, which produces small output hashing `u64`s and `u32`s
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        if cfg!(target_pointer_width = "64") {
            self.as_u64().hash(hasher);
        } else {
            self.line.hash(hasher);
            self.column.hash(hasher);
        }
    }
}

#[cfg(feature = "wasm")]
impl serde::Serialize for Position {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("line", &self.line)?;
        map.serialize_entry("column", &self.column)?;
        map.end()
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
