use std::str::FromStr;

pub fn validate_format(value: &str) -> bool {
    // URI-reference = URI / relative-ref (RFC 3986 §4.1)
    // A valid URI is also a valid URI-reference, so we check URI first.
    // For relative references, we check basic structural validity.
    tombi_uri::Uri::from_str(value).is_ok() || is_relative_reference(value)
}

fn is_relative_reference(value: &str) -> bool {
    // A relative reference must not start with a scheme (alpha + ":")
    // and must be a valid path with optional query and fragment.
    if value.is_empty() {
        return true; // empty string is a valid relative-ref
    }

    // Split off fragment
    let (without_fragment, fragment) = match value.split_once('#') {
        Some((before, after)) => (before, Some(after)),
        None => (value, None),
    };
    // Split off query
    let (path, query) = match without_fragment.split_once('?') {
        Some((before, after)) => (before, Some(after)),
        None => (without_fragment, None),
    };

    // Validate path, query, and fragment characters (RFC 3986)
    validate_uri_component(path)
        && query.map_or(true, validate_uri_component)
        && fragment.map_or(true, validate_uri_component)
}

fn validate_uri_component(component: &str) -> bool {
    let mut chars = component.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            // Percent-encoding must be followed by two hex digits (RFC 3986)
            match chars.next() {
                Some(ch) if ch.is_ascii_hexdigit() => {}
                _ => return false,
            }
            match chars.next() {
                Some(ch) if ch.is_ascii_hexdigit() => {}
                _ => return false,
            }
            continue;
        }

        if !(c.is_ascii_alphanumeric() || "-._~:@!$&'()*+,;=/".contains(c)) {
            return false;
        }
    }

    true
}
