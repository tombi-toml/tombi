use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Bound, Index, IndexMut, Range, RangeBounds, Sub, SubAssign},
};

use crate::{Offset, RawOffset, RawTextSize};

/// A span in text, represented as a pair of [`Offset`][struct@Offset].
///
/// It is a logic error for `start` to be greater than `end`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Span {
    // Invariant: start <= end
    start: Offset,
    end: Offset,
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start.raw, self.end.raw)
    }
}

impl Span {
    pub const MAX: Span = Span {
        start: Offset::MAX,
        end: Offset::MAX,
    };

    pub const MIN: Span = Span {
        start: Offset::MIN,
        end: Offset::MIN,
    };

    #[inline]
    pub const fn new(start: Offset, end: Offset) -> Span {
        assert!(start.raw <= end.raw);
        Span { start, end }
    }

    /// Create a new `Span` with the given `offset` and `len` (`offset..offset + len`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tombi_text::*;
    /// let text = "0123456789";
    ///
    /// let offset = Offset::from(2);
    /// let length = 5;
    /// let span = Span::at(offset, length);
    ///
    /// assert_eq!(span, Span::new(offset, offset + length));
    /// assert_eq!(&text[span], "23456")
    /// ```
    #[inline]
    pub const fn at(offset: Offset, len: RawOffset) -> Span {
        Span::new(offset, Offset::new(offset.raw + len))
    }

    /// Create a zero-length span at the specified offset (`offset..offset`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tombi_text::*;
    /// let point: Offset;
    /// # point = Offset::from(3);
    /// let span = Span::empty(point);
    /// assert!(span.is_empty());
    /// assert_eq!(span, Span::new(point, point));
    /// ```
    #[inline]
    pub const fn empty(offset: Offset) -> Span {
        Span {
            start: offset,
            end: offset,
        }
    }
}

/// Identity methods.
impl Span {
    /// The start point of this span.
    #[inline]
    pub const fn start(self) -> Offset {
        self.start
    }

    /// The end point of this span.
    #[inline]
    pub const fn end(self) -> Offset {
        self.end
    }

    /// The size of this span.
    #[inline]
    pub const fn len(self) -> RawOffset {
        // HACK for const fn: math on primitives only
        self.end.raw - self.start.raw
    }

    /// Check if this span is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        // HACK for const fn: math on primitives only
        self.start.raw == self.end.raw
    }
}

/// Manipulation methods.
impl Span {
    /// Check if this span contains an offset.
    ///
    /// The end index is considered excluded.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tombi_text::*;
    /// let (start, end): (Offset, Offset);
    /// # start = 10.into(); end = 20.into();
    /// let span = Span::new(start, end);
    /// assert!(span.contains(start));
    /// assert!(!span.contains(end));
    /// ```
    #[inline]
    pub fn contains(self, offset: Offset) -> bool {
        self.start <= offset && offset < self.end
    }

    /// Check if this span contains an offset.
    ///
    /// The end index is considered included.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tombi_text::*;
    /// let (start, end): (Offset, Offset);
    /// # start = 10.into(); end = 20.into();
    /// let span = Span::new(start, end);
    /// assert!(span.contains_inclusive(start));
    /// assert!(span.contains_inclusive(end));
    /// ```
    #[inline]
    pub fn contains_inclusive(self, offset: Offset) -> bool {
        self.start <= offset && offset <= self.end
    }

    /// Check if this span completely contains another span.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tombi_text::*;
    /// let larger = Span::new(0.into(), 20.into());
    /// let smaller = Span::new(5.into(), 15.into());
    /// assert!(larger.contains_span(smaller));
    /// assert!(!smaller.contains_span(larger));
    ///
    /// // a span always contains itself
    /// assert!(larger.contains_span(larger));
    /// assert!(smaller.contains_span(smaller));
    /// ```
    #[inline]
    pub fn contains_span(self, other: Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    /// The span covered by both ranges, if it exists.
    /// If the ranges touch but do not overlap, the output span is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tombi_text::*;
    /// assert_eq!(
    ///     Span::intersect(
    ///         Span::new(0.into(), 10.into()),
    ///         Span::new(5.into(), 15.into()),
    ///     ),
    ///     Some(Span::new(5.into(), 10.into())),
    /// );
    /// ```
    #[inline]
    pub fn intersect(self, other: Span) -> Option<Span> {
        let start = std::cmp::max(self.start, other.start);
        let end = std::cmp::min(self.end, other.end);
        if end < start {
            return None;
        }
        Some(Span::new(start, end))
    }

    /// Extends the span to cover `other` as well.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tombi_text::*;
    /// assert_eq!(
    ///     Span::cover(
    ///         Span::new(0.into(), 5.into()),
    ///         Span::new(15.into(), 20.into()),
    ///     ),
    ///     Span::new(0.into(), 20.into()),
    /// );
    /// ```
    #[inline]
    pub fn cover(self, other: Span) -> Span {
        let start = std::cmp::min(self.start, other.start);
        let end = std::cmp::max(self.end, other.end);
        Span::new(start, end)
    }

    /// Extends the span to cover `other` offsets as well.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tombi_text::*;
    /// assert_eq!(
    ///     Span::empty(0.into()).cover_offset(20.into()),
    ///     Span::new(0.into(), 20.into()),
    /// )
    /// ```
    #[inline]
    pub fn cover_offset(self, offset: Offset) -> Span {
        self.cover(Span::empty(offset))
    }

    /// Add an offset to this span.
    ///
    /// Note that this is not appropriate for changing where a `Span` is
    /// within some string; rather, it is for changing the reference anchor
    /// that the `Span` is measured against.
    ///
    /// The unchecked version (`Add::add`) will _always_ panic on overflow,
    /// in contrast to primitive integers, which check in debug mode only.
    #[inline]
    pub fn checked_add(self, offset: Offset) -> Option<Span> {
        Some(Span {
            start: self.start.checked_add(offset)?,
            end: self.end.checked_add(offset)?,
        })
    }

    /// Subtract an offset from this span.
    ///
    /// Note that this is not appropriate for changing where a `Span` is
    /// within some string; rather, it is for changing the reference anchor
    /// that the `Span` is measured against.
    ///
    /// The unchecked version (`Sub::sub`) will _always_ panic on overflow,
    /// in contrast to primitive integers, which check in debug mode only.
    #[inline]
    pub fn checked_sub(self, offset: Offset) -> Option<Span> {
        Some(Span {
            start: self.start.checked_sub(offset)?,
            end: self.end.checked_sub(offset)?,
        })
    }

    /// Relative order of the two ranges (overlapping ranges are considered
    /// equal).
    ///
    ///
    /// This is useful when, for example, binary searching an array of disjoint
    /// ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tombi_text::*;
    /// # use std::cmp::Ordering;
    ///
    /// let a = Span::new(0.into(), 3.into());
    /// let b = Span::new(4.into(), 5.into());
    /// assert_eq!(a.ordering(b), Ordering::Less);
    ///
    /// let a = Span::new(0.into(), 3.into());
    /// let b = Span::new(3.into(), 5.into());
    /// assert_eq!(a.ordering(b), Ordering::Less);
    ///
    /// let a = Span::new(0.into(), 3.into());
    /// let b = Span::new(2.into(), 5.into());
    /// assert_eq!(a.ordering(b), Ordering::Equal);
    ///
    /// let a = Span::new(0.into(), 3.into());
    /// let b = Span::new(2.into(), 2.into());
    /// assert_eq!(a.ordering(b), Ordering::Equal);
    ///
    /// let a = Span::new(2.into(), 3.into());
    /// let b = Span::new(2.into(), 2.into());
    /// assert_eq!(a.ordering(b), Ordering::Greater);
    /// ```
    #[inline]
    pub fn ordering(self, other: Span) -> Ordering {
        if self.end <= other.start {
            Ordering::Less
        } else if other.end <= self.start {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Index<Span> for str {
    type Output = str;
    #[inline]
    fn index(&self, index: Span) -> &str {
        &self[Range::<usize>::from(index)]
    }
}

impl Index<Span> for String {
    type Output = str;
    #[inline]
    fn index(&self, index: Span) -> &str {
        &self[Range::<usize>::from(index)]
    }
}

impl IndexMut<Span> for str {
    #[inline]
    fn index_mut(&mut self, index: Span) -> &mut str {
        &mut self[Range::<usize>::from(index)]
    }
}

impl IndexMut<Span> for String {
    #[inline]
    fn index_mut(&mut self, index: Span) -> &mut str {
        &mut self[Range::<usize>::from(index)]
    }
}

impl RangeBounds<RawTextSize> for Span {
    fn start_bound(&self) -> Bound<&RawTextSize> {
        Bound::Included(&self.start.raw)
    }

    fn end_bound(&self) -> Bound<&RawTextSize> {
        Bound::Excluded(&self.end.raw)
    }
}

impl From<(crate::RawTextSize, crate::RawTextSize)> for Span {
    #[inline]
    fn from((start, end): (crate::RawTextSize, crate::RawTextSize)) -> Self {
        Span::new(start.into(), end.into())
    }
}

impl<T> From<Span> for Range<T>
where
    T: From<Offset>,
{
    #[inline]
    fn from(r: Span) -> Self {
        r.start.into()..r.end.into()
    }
}

macro_rules! ops {
    (impl $Op:ident for Span by fn $f:ident = $op:tt) => {
        impl $Op<&Offset> for Span {
            type Output = Span;
            #[inline]
            fn $f(self, other: &Offset) -> Span {
                self $op *other
            }
        }
        impl<T> $Op<T> for &Span
        where
            Span: $Op<T, Output=Span>,
        {
            type Output = Span;
            #[inline]
            fn $f(self, other: T) -> Span {
                *self $op other
            }
        }
    };
}

impl Add<Offset> for Span {
    type Output = Span;
    #[inline]
    fn add(self, offset: Offset) -> Span {
        self.checked_add(offset).expect("Span +offset overflowed")
    }
}

impl Sub<Offset> for Span {
    type Output = Span;
    #[inline]
    fn sub(self, offset: Offset) -> Span {
        self.checked_sub(offset).expect("Span -offset overflowed")
    }
}

impl Add<Span> for Span {
    type Output = Span;
    #[inline]
    fn add(self, other: Span) -> Span {
        Span {
            start: self.start,
            end: other.end,
        }
    }
}

ops!(impl Add for Span by fn add = +);
ops!(impl Sub for Span by fn sub = -);

impl<A> AddAssign<A> for Span
where
    Span: Add<A, Output = Span>,
{
    #[inline]
    fn add_assign(&mut self, rhs: A) {
        *self = *self + rhs
    }
}

impl<S> SubAssign<S> for Span
where
    Span: Sub<S, Output = Span>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: S) {
        *self = *self - rhs
    }
}
