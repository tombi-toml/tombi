pub fn validate_format(value: &str) -> bool {
    addr::parse_domain_name(value).is_ok()
}
