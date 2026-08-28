//! SIMD-dispatched scanners for long ASCII JSON string content.
//!
//! Escape handling and validation stay scalar. This scanner only skips an
//! ordinary ASCII prefix; `memchr` selects vector implementations at runtime
//! where the target supports them.

pub(crate) const MIN_SIMD_INPUT_LEN: usize = 32;

#[inline]
fn is_ordinary_ascii_string_byte(byte: u8) -> bool {
    matches!(byte, 0x20..=0x7f) && !matches!(byte, b'"' | b'\\')
}

#[inline]
pub(crate) fn is_long_string_content(bytes: &[u8]) -> bool {
    bytes
        .get(..MIN_SIMD_INPUT_LEN)
        .is_some_and(|prefix| prefix.iter().copied().all(is_ordinary_ascii_string_byte))
}

#[inline]
pub(crate) fn ascii_before_quote_or_escape(bytes: &[u8]) -> usize {
    let end = memchr::memchr2(b'"', b'\\', bytes).unwrap_or(bytes.len());
    let prefix = &bytes[..end];
    prefix
        .iter()
        .copied()
        .position(|byte| !is_ordinary_ascii_string_byte(byte))
        .unwrap_or(end)
}
