pub fn validate_format(value: &str) -> bool {
    // RFC 6901: A JSON Pointer is either empty or starts with '/'
    // Each reference token is separated by '/'
    // '~' must be followed by '0' or '1' (escaped '~' and '/')
    if value.is_empty() {
        return true;
    }

    if !value.starts_with('/') {
        return false;
    }

    let mut chars = value[1..].chars();
    while let Some(c) = chars.next() {
        if c == '~' {
            match chars.next() {
                Some('0') | Some('1') => {}
                _ => return false, // invalid escape
            }
        }
    }

    true
}
