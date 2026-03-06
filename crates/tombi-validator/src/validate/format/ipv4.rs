use std::net::Ipv4Addr;

pub fn validate_format(value: &str) -> bool {
    value.parse::<Ipv4Addr>().is_ok()
}
