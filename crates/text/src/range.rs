use std::ops::AddAssign;

use crate::{Column, Line, Position, RelativePosition};

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Range {
    // Invariant: start <= end
    start: Position,
    end: Position,
}

impl Range {
    #[inline]
    pub fn new(start: Position, end: Position) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self {
            start: Position::zero(),
            end: Position::zero(),
        }
    }

    #[inline]
    pub fn start(&self) -> Position {
        self.start
    }

    #[inline]
    pub fn end(&self) -> Position {
        self.end
    }
}

impl std::fmt::Debug for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
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

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;
    use std::cmp::Ordering;

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
        dbg!(&r1, &r2);
        assert_eq!(r1.cmp(&r2), expected);
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
        range += text.into();
        assert_eq!(range, expected.into());
    }
}
