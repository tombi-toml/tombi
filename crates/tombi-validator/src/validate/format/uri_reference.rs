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
    let without_fragment = value.split('#').next().unwrap_or("");
    // Split off query
    let path = without_fragment.split('?').next().unwrap_or("");

    // Basic validation: path characters must be valid
    path.chars()
        .all(|c| c.is_ascii_alphanumeric() || "-._~:@!$&'()*+,;=/".contains(c) || c == '%')
}
