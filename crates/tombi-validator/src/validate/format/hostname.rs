pub fn validate(value: &str) -> bool {
    addr::parse_domain_name(value).is_ok()
}
