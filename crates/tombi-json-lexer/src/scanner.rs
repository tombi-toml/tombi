//! SIMD scanner for ordinary JSON string bytes.

pub(crate) const MIN_SIMD_INPUT_LEN: usize = 32;

#[inline]
fn is_ordinary_ascii_string_byte(byte: u8) -> bool {
    matches!(byte, 0x20..=0x7f) && !matches!(byte, b'"' | b'\\')
}

#[inline]
pub(crate) fn ordinary_ascii_prefix(bytes: &[u8]) -> usize {
    if bytes.len() < MIN_SIMD_INPUT_LEN {
        return bytes
            .iter()
            .position(|&byte| !is_ordinary_ascii_string_byte(byte))
            .unwrap_or(bytes.len());
    }

    #[cfg(target_arch = "aarch64")]
    // SAFETY: AArch64 always provides NEON. The scanner only performs
    // unaligned reads within `bytes` and uses the mask to locate a boundary.
    unsafe {
        ordinary_ascii_prefix_neon(bytes)
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        let prefix = &bytes[..MIN_SIMD_INPUT_LEN];
        if let Some(index) = prefix
            .iter()
            .position(|&byte| !is_ordinary_ascii_string_byte(byte))
        {
            return index;
        }

        let end = memchr::memchr2(b'"', b'\\', bytes).unwrap_or(bytes.len());
        bytes[..end]
            .iter()
            .position(|&byte| !is_ordinary_ascii_string_byte(byte))
            .unwrap_or(end)
    }
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn ordinary_ascii_prefix_neon(bytes: &[u8]) -> usize {
    use core::arch::aarch64::*;

    let quote = vdupq_n_u8(b'"');
    let escape = vdupq_n_u8(b'\\');
    let control_limit = vdupq_n_u8(0x20);
    let non_ascii_limit = vdupq_n_u8(0x80);
    let mut offset = 0;

    while offset + 16 <= bytes.len() {
        // SAFETY: The loop condition guarantees that 16 bytes are available.
        let chunk = unsafe { vld1q_u8(bytes.as_ptr().add(offset)) };
        let quote_mask = vceqq_u8(chunk, quote);
        let escape_mask = vceqq_u8(chunk, escape);
        let control_mask = vcltq_u8(chunk, control_limit);
        let non_ascii_mask = vcgeq_u8(chunk, non_ascii_limit);
        let special_mask = vorrq_u8(
            vorrq_u8(quote_mask, escape_mask),
            vorrq_u8(control_mask, non_ascii_mask),
        );

        if vmaxvq_u8(special_mask) != 0 {
            let mut lanes = [0u8; 16];
            // SAFETY: `lanes` has exactly 16 writable bytes.
            unsafe { vst1q_u8(lanes.as_mut_ptr(), special_mask) };
            return offset
                + lanes
                    .iter()
                    .position(|&lane| lane != 0)
                    .expect("a non-zero SIMD mask must contain a non-zero lane");
        }

        offset += 16;
    }

    offset
        + bytes[offset..]
            .iter()
            .position(|&byte| !is_ordinary_ascii_string_byte(byte))
            .unwrap_or(bytes.len() - offset)
}
