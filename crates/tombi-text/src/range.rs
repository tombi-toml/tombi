use std::ops::{Add, AddAssign};

use crate::{Column, Line, Position, RelativePosition};

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Range {
    // Invariant: start <= end
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub const MAX: Range = Range {
        start: Position::MAX,
        end: Position::MAX,
    };
    pub const MIN: Range = Range {
        start: Position::MIN,
        end: Position::MIN,
    };

    #[inline]
    pub fn new(start: Position, end: Position) -> Self {
        // assert!(start <= end);
        Self {
            start,
            end: if start <= end {
                end
            } else {
                tracing::error!(
                    "Invalid tombi_text::Range: start: {:?} > end: {:?}",
                    start,
                    end
                );
                start
            },
        }
    }
    /// New range with 1-based line and column
    #[inline]
    pub fn new_1_based(range: Self) -> Self {
        Self::new(
            Position::new(range.start.line + 1, range.start.column + 1),
            Position::new(range.end.line + 1, range.end.column + 1),
        )
    }

    #[inline]
    pub fn at(position: Position) -> Self {
        Self::new(position, position)
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.start.line == self.end.line && self.start.column == self.end.column
    }

    #[inline]
    pub fn contains(&self, position: Position) -> bool {
        self.start <= position && position <= self.end
    }
}

impl std::fmt::Debug for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Ord for Range {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.start.cmp(&other.start) {
            std::cmp::Ordering::Equal => self.end.cmp(&other.end),
            ord => ord,
        }
    }
}

impl PartialOrd for Range {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<(Position, Position)> for Range {
    #[inline]
    fn from((start, end): (Position, Position)) -> Self {
        Self::new(start, end)
    }
}

impl From<((Line, Column), (Line, Column))> for Range {
    #[inline]
    fn from(
        ((start_line, start_column), (end_line, end_column)): ((Line, Column), (Line, Column)),
    ) -> Self {
        Self::new(
            Position::new(start_line, start_column),
            Position::new(end_line, end_column),
        )
    }
}

impl AddAssign<RelativePosition> for Range {
    #[inline]
    fn add_assign(&mut self, rhs: RelativePosition) {
        self.end += rhs;
    }
}

impl AddAssign for Range {
    #[inline]
    fn add_assign(&mut self, rhs: Range) {
        *self = Range::new(
            std::cmp::min(self.start, rhs.start),
            std::cmp::max(self.end, rhs.end),
        );
    }
}

impl Add<Range> for Range {
    type Output = Range;

    #[inline]
    fn add(self, rhs: Range) -> Self::Output {
        Range::new(
            std::cmp::min(self.start, rhs.start),
            std::cmp::max(self.end, rhs.end),
        )
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(((1, 1), (1, 2)), ((1, 1), (1, 2)), Ordering::Equal)]
    #[case(((1, 1), (1, 2)), ((1, 1), (1, 3)), Ordering::Less)]
    #[case(((1, 1), (1, 2)), ((1, 2), (1, 2)), Ordering::Less)]
    #[case(((1, 1), (1, 2)), ((1, 2), (1, 3)), Ordering::Less)]
    #[case(((1, 1), (1, 2)), ((2, 1), (2, 2)), Ordering::Less)]
    #[case(((1, 1), (1, 2)), ((1, 1), (1, 1)), Ordering::Greater)]
    #[case(((1, 1), (2, 1)), ((1, 1), (1, 1)), Ordering::Greater)]
    fn test_range_cmp(
        #[case] range: ((Line, Column), (Line, Column)),
        #[case] other: ((Line, Column), (Line, Column)),
        #[case] expected: Ordering,
    ) {
        let r1 = Range::from(range);
        let r2 = Range::from(other);

        pretty_assertions::assert_eq!(r1.cmp(&r2), expected);
    }

    #[rstest]
    #[case(((1, 1), (1, 2)), "a", ((1, 1), (1, 3)))]
    #[case(((1, 1), (1, 2)), "a\n", ((1, 1), (2, 0)))]
    #[case(((1, 1), (1, 2)), "a\nb", ((1, 1), (2, 1)))]
    fn test_add_assign(
        #[case] range: ((Line, Column), (Line, Column)),
        #[case] text: &str,
        #[case] expected: ((Line, Column), (Line, Column)),
    ) {
        let mut range = Range::from(range);
        range += RelativePosition::of(text);
        pretty_assertions::assert_eq!(range, expected.into());
    }
}
