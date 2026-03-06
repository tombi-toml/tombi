pub fn validate_format(value: &str) -> bool {
    // RFC 3339 full-time: hh:mm:ss[.frac](Z | +hh:mm | -hh:mm)
    let Some(rest) = parse_time_components(value) else {
        return false;
    };

    // Time zone: Z, +hh:mm, or -hh:mm
    validate_timezone(rest)
}

/// Parse the time components (HH:MM:SS[.frac]) and return the remaining string.
/// Returns `None` if the input is not a valid time component.
pub(crate) fn parse_time_components(value: &str) -> Option<&str> {
    if value.len() < 8 || !value.is_ascii() {
        return None;
    }

    let bytes = value.as_bytes();
    if bytes[2] != b':' || bytes[5] != b':' {
        return None;
    }

    let Ok(hour) = value[0..2].parse::<u32>() else {
        return None;
    };
    let Ok(minute) = value[3..5].parse::<u32>() else {
        return None;
    };
    let Ok(second) = value[6..8].parse::<u32>() else {
        return None;
    };

    if hour > 23 || minute > 59 || second > 60 {
        // 60 seconds allowed for leap second
        return None;
    }

    let rest = &value[8..];

    // Optional fractional seconds
    if let Some(stripped) = rest.strip_prefix('.') {
        let frac_end = stripped
            .bytes()
            .position(|b| !b.is_ascii_digit())
            .unwrap_or(stripped.len());
        if frac_end == 0 {
            return None; // "." with no digits
        }
        Some(&stripped[frac_end..])
    } else {
        Some(rest)
    }
}

/// Validate a timezone suffix (Z, +hh:mm, or -hh:mm).
pub(crate) fn validate_timezone(rest: &str) -> bool {
    if !rest.is_ascii() {
        return false;
    }
    match rest {
        "Z" | "z" => true,
        s if (s.starts_with('+') || s.starts_with('-')) && s.len() == 6 => {
            let tz = &s[1..];
            tz.as_bytes()[2] == b':'
                && tz[0..2].parse::<u32>().is_ok_and(|h| h <= 23)
                && tz[3..5].parse::<u32>().is_ok_and(|m| m <= 59)
        }
        _ => false,
    }
}
