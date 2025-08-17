use std::str::FromStr;

pub fn validate_format(value: &str) -> bool {
    tombi_uri::Uri::from_str(value).is_ok()
}
