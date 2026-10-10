use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

/// Why the policy refused a schema or catalog access.
///
/// Host refusals are typed so callers can build their own message instead of
/// matching on text.
#[derive(Debug, Clone, thiserror::Error)]
pub(crate) enum PolicyError {
    #[error("destination host is not trusted")]
    HostNotTrusted,

    #[error("schema access denied: {reason}")]
    Denied { reason: String },
}

impl PolicyError {
    fn new(reason: impl Into<String>) -> Self {
        Self::Denied {
            reason: reason.into(),
        }
    }
}

/// Security policy for schema and catalog retrieval.
///
/// Trust is loaded from the user's process environment. Project configuration
/// cannot add network grants.
#[derive(Debug, Clone, Default)]
pub(crate) struct SchemaFetchPolicy {
    /// Normalized with `normalize_host`.
    trusted_hosts: Vec<String>,
    configuration_error: Option<String>,
}

impl SchemaFetchPolicy {
    /// Build the policy from the process environment plus the trusted hosts
    /// supplied by the embedding application through [`crate::Options`].
    ///
    /// Options entries are added to, not substituted for, the environment.
    pub(crate) fn from_options(options: &crate::Options) -> Self {
        let mut policy = Self::default();
        let mut configuration_errors = Vec::new();

        let mut hosts: Vec<String> = Vec::new();
        if let Some(value) = std::env::var_os("TOMBI_SCHEMA_TRUSTED_HOSTS") {
            hosts.extend(
                value
                    .to_string_lossy()
                    .split(',')
                    .map(str::trim)
                    .filter(|entry| !entry.is_empty())
                    .map(str::to_string),
            );
        }
        hosts.extend(options.trusted_hosts.iter().flatten().cloned());
        for entry in &hosts {
            match parse_trusted_host(entry) {
                Ok(host) => policy.trusted_hosts.push(normalize_host(&host)),
                Err(reason) => configuration_errors.push(reason),
            }
        }

        if !configuration_errors.is_empty() {
            let reason = configuration_errors.join("; ");
            log::warn!("invalid schema trust configuration: {reason}");
            policy.configuration_error = Some(reason);
        }

        policy
    }

    /// Reject unsupported or credential-bearing URLs before cache lookup.
    pub(crate) fn check_http_url(&self, url: &tombi_uri::Uri) -> Result<(), PolicyError> {
        self.check_configuration()?;

        if !matches!(url.scheme(), "http" | "https") {
            return Err(PolicyError::new(
                "only HTTP and HTTPS schema URLs are allowed",
            ));
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(PolicyError::new("URL credentials are not allowed"));
        }

        let address = match url.host() {
            Some(tombi_uri::Host::Ipv4(address)) => IpAddr::V4(address),
            Some(tombi_uri::Host::Ipv6(address)) => IpAddr::V6(address),
            Some(tombi_uri::Host::Domain(_)) => return Ok(()),
            None => return Err(PolicyError::new("URL has no host")),
        };
        if !self.address_is_allowed(&address.to_string(), address) {
            return Err(PolicyError::HostNotTrusted);
        }

        Ok(())
    }

    /// Validate the concrete DNS answers that reqwest will connect to.
    /// Mixed public/private answers are rejected as a whole.
    #[cfg(any(test, all(feature = "reqwest", not(target_arch = "wasm32"))))]
    pub(crate) fn check_resolved_addresses(
        &self,
        host: &str,
        addresses: &[IpAddr],
    ) -> Result<(), PolicyError> {
        self.check_configuration()?;

        if addresses.is_empty() {
            return Err(PolicyError::new("DNS returned no addresses"));
        }

        for address in addresses {
            if !self.address_is_allowed(host, *address) {
                return Err(PolicyError::HostNotTrusted);
            }
        }

        Ok(())
    }

    fn address_is_allowed(&self, host: &str, address: IpAddr) -> bool {
        is_globally_reachable(address) || {
            let host = normalize_host(host);
            self.trusted_hosts.contains(&host)
        }
    }

    fn check_configuration(&self) -> Result<(), PolicyError> {
        if let Some(reason) = &self.configuration_error {
            return Err(PolicyError::new(format!(
                "invalid user schema trust configuration: {reason}"
            )));
        }
        Ok(())
    }
}

fn parse_trusted_host(entry: &str) -> Result<String, String> {
    let bracketed_ipv6 = entry.starts_with('[') && entry.ends_with(']');
    if entry.contains('@')
        || entry.contains('/')
        || entry.contains('?')
        || entry.contains('#')
        || (entry.contains(':') && !bracketed_ipv6)
    {
        return Err(format!(
            "`{entry}` must be a hostname or IP address without credentials, port, or path"
        ));
    }

    let url = tombi_uri::Uri::from_str(&format!("https://{entry}/"))
        .map_err(|_| format!("`{entry}` must be a hostname or IP address"))?;
    if url.port().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(format!(
            "`{entry}` must be a hostname or IP address without credentials, port, or path"
        ));
    }

    match url.host() {
        Some(tombi_uri::Host::Domain(host)) => Ok(host.to_string()),
        Some(tombi_uri::Host::Ipv4(address)) => Ok(address.to_string()),
        Some(tombi_uri::Host::Ipv6(address)) => Ok(address.to_string()),
        None => Err(format!("`{entry}` has no hostname or IP address")),
    }
}

fn normalize_host(host: &str) -> String {
    host.trim_end_matches('.').to_ascii_lowercase()
}

fn is_globally_reachable(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_globally_reachable_v4(address),
        IpAddr::V6(address) => is_globally_reachable_v6(address),
    }
}

fn is_globally_reachable_v4(address: Ipv4Addr) -> bool {
    const BLOCKED: &[(Ipv4Addr, u8)] = &[
        (Ipv4Addr::new(0, 0, 0, 0), 8),
        (Ipv4Addr::new(10, 0, 0, 0), 8),
        (Ipv4Addr::new(100, 64, 0, 0), 10),
        (Ipv4Addr::new(127, 0, 0, 0), 8),
        (Ipv4Addr::new(169, 254, 0, 0), 16),
        (Ipv4Addr::new(172, 16, 0, 0), 12),
        (Ipv4Addr::new(192, 0, 0, 0), 24),
        (Ipv4Addr::new(192, 0, 2, 0), 24),
        (Ipv4Addr::new(192, 88, 99, 0), 24),
        (Ipv4Addr::new(192, 168, 0, 0), 16),
        (Ipv4Addr::new(198, 18, 0, 0), 15),
        (Ipv4Addr::new(198, 51, 100, 0), 24),
        (Ipv4Addr::new(203, 0, 113, 0), 24),
        (Ipv4Addr::new(224, 0, 0, 0), 4),
        (Ipv4Addr::new(240, 0, 0, 0), 4),
    ];

    !BLOCKED
        .iter()
        .any(|(network, prefix)| ipv4_in_network(address, *network, *prefix))
}

fn is_globally_reachable_v6(address: Ipv6Addr) -> bool {
    const BLOCKED: &[(Ipv6Addr, u8)] = &[
        (Ipv6Addr::new(0x2001, 0, 0, 0, 0, 0, 0, 0), 23),
        (Ipv6Addr::new(0x2001, 0x10, 0, 0, 0, 0, 0, 0), 28),
        (Ipv6Addr::new(0x2001, 0x20, 0, 0, 0, 0, 0, 0), 28),
        (Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32),
        (Ipv6Addr::new(0x2002, 0, 0, 0, 0, 0, 0, 0), 16),
        (Ipv6Addr::new(0x3fff, 0, 0, 0, 0, 0, 0, 0), 20),
    ];
    let segments = address.segments();
    let is_global_unicast = segments[0] & 0xe000 == 0x2000;

    is_global_unicast
        && !BLOCKED
            .iter()
            .any(|(network, prefix)| ipv6_in_network(address, *network, *prefix))
}

fn ipv4_in_network(address: Ipv4Addr, network: Ipv4Addr, prefix: u8) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    u32::from(address) & mask == u32::from(network) & mask
}

fn ipv6_in_network(address: Ipv6Addr, network: Ipv6Addr, prefix: u8) -> bool {
    let address = u128::from(address);
    let network = u128::from(network);
    let mask = if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    };
    address & mask == network & mask
}

#[cfg(test)]
mod tests {
    use std::{net::IpAddr, str::FromStr};

    use rstest::rstest;

    use super::{SchemaFetchPolicy, is_globally_reachable};

    #[rstest]
    #[case("8.8.8.8", true)]
    #[case("1.1.1.1", true)]
    #[case("0.0.0.0", false)]
    #[case("10.0.0.1", false)]
    #[case("100.64.0.1", false)]
    #[case("127.0.0.1", false)]
    #[case("169.254.169.254", false)]
    #[case("172.16.0.1", false)]
    #[case("192.0.2.1", false)]
    #[case("192.168.1.1", false)]
    #[case("198.18.0.1", false)]
    #[case("198.51.100.1", false)]
    #[case("203.0.113.1", false)]
    #[case("224.0.0.1", false)]
    #[case("240.0.0.1", false)]
    #[case("2606:4700:4700::1111", true)]
    #[case("::", false)]
    #[case("::1", false)]
    #[case("fc00::1", false)]
    #[case("fe80::1", false)]
    #[case("2001:db8::1", false)]
    #[case("2002:0808:0808::1", false)]
    #[case("3fff::1", false)]
    fn address_reachability(#[case] address: &str, #[case] expected: bool) {
        assert_eq!(
            is_globally_reachable(IpAddr::from_str(address).unwrap()),
            expected
        );
    }

    #[test]
    fn mixed_dns_answers_are_denied() {
        let policy = SchemaFetchPolicy::default();
        let addresses = [
            IpAddr::from_str("8.8.8.8").unwrap(),
            IpAddr::from_str("127.0.0.1").unwrap(),
        ];

        assert!(
            policy
                .check_resolved_addresses("schema.example", &addresses)
                .is_err()
        );
    }

    #[test]
    fn a_later_dns_answer_is_checked_again_for_rebinding() {
        let policy = SchemaFetchPolicy::default();
        let public = [IpAddr::from_str("8.8.8.8").unwrap()];
        let rebound = [IpAddr::from_str("127.0.0.1").unwrap()];

        assert!(
            policy
                .check_resolved_addresses("schema.example", &public)
                .is_ok()
        );
        assert!(
            policy
                .check_resolved_addresses("schema.example", &rebound)
                .is_err()
        );
    }

    #[test]
    fn http_urls_reject_credentials_and_local_literals() {
        let policy = SchemaFetchPolicy::default();
        let local = tombi_uri::Uri::from_str("http://127.0.0.1/schema.json").unwrap();
        let credentials =
            tombi_uri::Uri::from_str("https://user:pass@example.com/schema.json").unwrap();
        let public = tombi_uri::Uri::from_str("https://example.com/schema.json").unwrap();

        assert!(policy.check_http_url(&local).is_err());
        assert!(policy.check_http_url(&credentials).is_err());
        assert!(policy.check_http_url(&public).is_ok());
    }

    #[test]
    fn trusted_hosts_reject_credentials_and_ports() {
        assert!(super::parse_trusted_host("user@schema.example").is_err());
        assert!(super::parse_trusted_host("schema.example:443").is_err());
        assert_eq!(super::parse_trusted_host("[fd00::1]").unwrap(), "fd00::1");
    }

    #[test]
    fn trust_configuration_is_user_scoped() {
        let policy = SchemaFetchPolicy {
            trusted_hosts: vec!["schema.internal".to_string()],
            configuration_error: None,
        };
        let private = IpAddr::from_str("10.1.2.3").unwrap();

        assert!(
            policy
                .check_resolved_addresses("schema.internal", &[private])
                .is_ok()
        );
        assert!(
            policy
                .check_resolved_addresses("other.internal", &[private])
                .is_err()
        );
    }

    #[test]
    fn options_extend_the_trusted_hosts() {
        let private = IpAddr::from_str("10.1.2.3").unwrap();

        let policy = SchemaFetchPolicy::from_options(&crate::Options {
            trusted_hosts: Some(vec!["schema.internal".to_string()]),
            ..Default::default()
        });

        assert!(
            policy
                .check_resolved_addresses("schema.internal", &[private])
                .is_ok()
        );
        assert!(matches!(
            policy.check_resolved_addresses("other.internal", &[private]),
            Err(super::PolicyError::HostNotTrusted)
        ));
    }

    #[test]
    fn invalid_option_entries_refuse_every_access() {
        let policy = SchemaFetchPolicy::from_options(&crate::Options {
            trusted_hosts: Some(vec!["user@schema.example".to_string()]),
            ..Default::default()
        });
        let public = tombi_uri::Uri::from_str("https://example.com/schema.json").unwrap();

        assert!(matches!(
            policy.check_http_url(&public),
            Err(super::PolicyError::Denied { .. })
        ));
    }
}
