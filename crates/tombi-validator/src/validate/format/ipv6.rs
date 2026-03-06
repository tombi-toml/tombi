use std::net::Ipv6Addr;

pub fn validate_format(value: &str) -> bool {
    value.parse::<Ipv6Addr>().is_ok()
}
