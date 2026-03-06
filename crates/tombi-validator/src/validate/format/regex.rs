pub fn validate_format(value: &str) -> bool {
    tombi_regex::Regex::new(value).is_ok()
}
