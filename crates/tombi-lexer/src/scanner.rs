//! SIMD-dispatched ASCII scanners for long lexer runs.
//!
//! Grammar and recovery stay scalar. These helpers only locate bytes that
//! require grammar-aware handling; `memchr` selects vector implementations at
//! runtime where the target supports them.

pub(crate) const MIN_SIMD_INPUT_LEN: usize = 32;

#[inline]
pub(crate) fn is_long_line(bytes: &[u8]) -> bool {
    if bytes.len() < MIN_SIMD_INPUT_LEN {
        return false;
    }
    let prefix = &bytes[..MIN_SIMD_INPUT_LEN];
    memchr::memchr2(b'\r', b'\n', prefix).is_none() && prefix.is_ascii()
}

#[inline]
fn ascii_before(bytes: &[u8], special: Option<usize>) -> usize {
    let end = special.unwrap_or(bytes.len());
    let prefix = &bytes[..end];
    if prefix.is_ascii() {
        end
    } else {
        prefix
            .iter()
            .position(|byte| !byte.is_ascii())
            .expect("a non-ASCII prefix must contain a non-ASCII byte")
    }
}

#[inline]
pub(crate) fn ascii_before_line_break(bytes: &[u8]) -> usize {
    ascii_before(bytes, memchr::memchr2(b'\r', b'\n', bytes))
}
