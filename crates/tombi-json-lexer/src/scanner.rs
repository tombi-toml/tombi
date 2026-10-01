//! SIMD scanner for ordinary JSON string bytes.

const MIN_SIMD_INPUT_LEN: usize = 32;

/// Returns whether `byte` is part of a string without special handling.
///
/// Non-ASCII bytes are ordinary: the string content is scanned as bytes, and a multi-byte
/// char never contains a quote, a backslash or a control byte.
#[inline]
fn is_ordinary_string_byte(byte: u8) -> bool {
    byte >= 0x20 && byte != b'"' && byte != b'\\'
}

/// Returns the length of the prefix of `bytes` that has no quote, backslash or control byte.
#[inline]
pub(crate) fn ordinary_string_prefix(bytes: &[u8]) -> usize {
    if bytes.len() < MIN_SIMD_INPUT_LEN {
        return scalar_ordinary_string_prefix(bytes);
    }

    #[cfg(target_arch = "aarch64")]
    // SAFETY: AArch64 always provides NEON. The scanner only performs
    // unaligned reads within `bytes` and uses the mask to locate a boundary.
    unsafe {
        ordinary_string_prefix_neon(bytes)
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        let prefix = &bytes[..MIN_SIMD_INPUT_LEN];
        if let Some(index) = prefix
            .iter()
            .position(|&byte| !is_ordinary_string_byte(byte))
        {
            return index;
        }

        let end = memchr::memchr2(b'"', b'\\', bytes).unwrap_or(bytes.len());
        scalar_ordinary_string_prefix(&bytes[..end])
    }
}

#[inline]
fn scalar_ordinary_string_prefix(bytes: &[u8]) -> usize {
    bytes
        .iter()
        .position(|&byte| !is_ordinary_string_byte(byte))
        .unwrap_or(bytes.len())
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn ordinary_string_prefix_neon(bytes: &[u8]) -> usize {
    use core::arch::aarch64::*;

    let quote = vdupq_n_u8(b'"');
    let escape = vdupq_n_u8(b'\\');
    let control_limit = vdupq_n_u8(0x20);
    let mut offset = 0;

    while offset + 16 <= bytes.len() {
        // SAFETY: The loop condition guarantees that 16 bytes are available.
        let chunk = unsafe { vld1q_u8(bytes.as_ptr().add(offset)) };
        let special_mask = vorrq_u8(
            vorrq_u8(vceqq_u8(chunk, quote), vceqq_u8(chunk, escape)),
            vcltq_u8(chunk, control_limit),
        );

        // Narrow each 8-bit lane to 4 bits, so that the mask fits in a `u64`.
        let nibbles = vshrn_n_u16::<4>(vreinterpretq_u16_u8(special_mask));
        let mask = vget_lane_u64::<0>(vreinterpret_u64_u8(nibbles));
        if mask != 0 {
            return offset + (mask.trailing_zeros() / 4) as usize;
        }

        offset += 16;
    }

    offset + scalar_ordinary_string_prefix(&bytes[offset..])
}
