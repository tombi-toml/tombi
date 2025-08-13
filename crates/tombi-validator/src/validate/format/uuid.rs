pub fn validate_format(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok()
}
