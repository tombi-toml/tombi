use std::{
    convert::TryFrom,
    fmt, iter,
    num::TryFromIntError,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::RawTextSize;

/// A measure of text length. Also, equivalently, an index into text.
///
/// This is a UTF-8 bytes offset stored as `u32`, but
/// most clients should treat it as an opaque measure.
///
/// For cases that need to escape `Offset` and return to working directly
/// with primitive integers, `Offset` can be converted losslessly to/from
/// `u32` via [`From`] conversions as well as losslessly be converted [`Into`]
/// `usize`. The `usize -> Offset` direction can be done via [`TryFrom`].
///
/// These escape hatches are primarily required for unit testing and when
/// converting from UTF-8 size to another coordinate space, such as UTF-16.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Offset {
    pub(crate) raw: RawTextSize,
}

impl fmt::Debug for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl Offset {
    pub const MAX: Offset = Offset {
        raw: RawTextSize::MAX,
    };

    pub const MIN: Offset = Offset { raw: 0 };

    /// Creates a new instance of `Offset` from a raw `u32`.
    #[inline]
    pub const fn new(raw: u32) -> Offset {
        Offset { raw }
    }

    /// The text size of some primitive text-like object.
    ///
    /// Accepts `char`, `&str`, and `&String`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use tombi_text::*;
    /// let char_size = Offset::of("🦀");
    /// assert_eq!(char_size, Offset::from(4));
    ///
    /// let str_size = Offset::of("rust-analyzer");
    /// assert_eq!(str_size, Offset::from(13));
    /// ```
    #[inline]
    pub fn of(text: &str) -> Offset {
        Self::new(text.len() as RawTextSize)
    }
}

/// Methods to act like a primitive integer type, where reasonably applicable.
//  Last updated for parity with Rust 1.42.0.
impl Offset {
    /// Checked addition. Returns `None` if overflow occurred.
    #[inline]
    pub const fn checked_add(self, rhs: Offset) -> Option<Offset> {
        match self.raw.checked_add(rhs.raw) {
            Some(raw) => Some(Offset { raw }),
            None => None,
        }
    }

    /// Checked subtraction. Returns `None` if overflow occurred.
    #[inline]
    pub const fn checked_sub(self, rhs: Offset) -> Option<Offset> {
        match self.raw.checked_sub(rhs.raw) {
            Some(raw) => Some(Offset { raw }),
            None => None,
        }
    }
}

impl From<u32> for Offset {
    #[inline]
    fn from(raw: u32) -> Self {
        Offset { raw }
    }
}

impl From<Offset> for u32 {
    #[inline]
    fn from(value: Offset) -> Self {
        value.raw
    }
}

impl TryFrom<usize> for Offset {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(value: usize) -> Result<Self, TryFromIntError> {
        Ok(u32::try_from(value)?.into())
    }
}

impl From<Offset> for usize {
    #[inline]
    fn from(value: Offset) -> Self {
        value.raw as usize
    }
}

impl Add<crate::RawOffset> for Offset {
    type Output = Offset;

    #[inline]
    fn add(self, rhs: crate::RawOffset) -> Self::Output {
        Offset::new(self.raw + rhs)
    }
}

impl Add for Offset {
    type Output = Offset;

    #[inline]
    fn add(self, rhs: crate::Offset) -> Self::Output {
        Offset::new(self.raw + rhs.raw)
    }
}

impl Sub for Offset {
    type Output = Offset;

    #[inline]
    fn sub(self, rhs: crate::Offset) -> Self::Output {
        Offset::new(self.raw - rhs.raw)
    }
}

impl Sub<crate::RawOffset> for Offset {
    type Output = Offset;

    #[inline]
    fn sub(self, rhs: crate::RawOffset) -> Self::Output {
        Offset::new(self.raw - rhs)
    }
}

impl<A> AddAssign<A> for Offset
where
    Offset: Add<A, Output = Offset>,
{
    #[inline]
    fn add_assign(&mut self, rhs: A) {
        *self = *self + rhs
    }
}

impl<S> SubAssign<S> for Offset
where
    Offset: Sub<S, Output = Offset>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: S) {
        *self = *self - rhs
    }
}

impl<A> iter::Sum<A> for Offset
where
    Offset: Add<A, Output = Offset>,
{
    #[inline]
    fn sum<I: Iterator<Item = A>>(iter: I) -> Offset {
        iter.fold(0.into(), Add::add)
    }
}
