pub fn validate(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok()
}
