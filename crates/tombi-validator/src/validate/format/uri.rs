pub fn validate(value: &str) -> bool {
    url::Url::parse(value).is_ok()
}
