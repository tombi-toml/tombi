pub fn validate_format(value: &str) -> bool {
    url::Url::parse(value).is_ok()
}
