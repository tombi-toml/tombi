use std::{net::IpAddr, str::FromStr};

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
/// Destination-address filtering is disabled: any host or IP address is accepted.
/// URL scheme (`http`/`https`) and credential checks are still enforced.
/// `trusted_hosts` is still parsed for compatibility but has no effect.
#[derive(Debug, Clone, Default)]
pub(crate) struct SchemaFetchPolicy {
    /// Normalized with `normalize_host`.
    trusted_hosts: Vec<String>,
}

impl SchemaFetchPolicy {
    /// Build the policy from the process environment plus the trusted hosts
    /// supplied by the embedding application through [`crate::Options`].
    ///
    /// Options entries are added to, not substituted for, the environment.
    /// Because address filtering is disabled, invalid entries are logged and skipped.
    pub(crate) fn from_options(options: &crate::Options) -> Self {
        let mut policy = Self::default();

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
                Err(reason) => log::warn!("invalid schema trust configuration: {reason}"),
            }
        }

        policy
    }

    /// Reject unsupported or credential-bearing URLs before cache lookup.
    ///
    /// Address filtering is disabled: any host, private or public, is accepted.
    /// Scheme and credential checks are retained.
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
    ///
    /// Address filtering is disabled, so any non-empty set of answers is accepted.
    /// Private and mixed public/private answers are included.
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

    /// Configuration is always considered valid.
    /// Retained so the policy call graph and its callers stay unchanged.
    fn check_configuration(&self) -> Result<(), PolicyError> {
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

/// Whether an address is allowed as a schema destination.
///
/// Address filtering is disabled, so every address is accepted.
/// The parameter is retained so the policy call graph stays intact.
fn is_globally_reachable(_address: IpAddr) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use std::{net::IpAddr, str::FromStr};

    use super::SchemaFetchPolicy;

    #[test]
    fn private_ip_literals_are_accepted() {
        let policy = SchemaFetchPolicy::default();
        for host in ["10.1.2.3", "127.0.0.1"] {
            let uri = tombi_uri::Uri::from_str(&format!("http://{host}/schema.json")).unwrap();

            assert!(
                policy.check_http_url(&uri).is_ok(),
                "{host} should be accepted"
            );
        }
    }

    #[test]
    fn benchmark_and_mixed_dns_answers_are_accepted() {
        let policy = SchemaFetchPolicy::default();

        let benchmark = IpAddr::from_str("198.18.0.1").unwrap();
        assert!(
            policy
                .check_resolved_addresses("schema.example", &[benchmark])
                .is_ok()
        );

        let mixed = [
            IpAddr::from_str("8.8.8.8").unwrap(),
            IpAddr::from_str("127.0.0.1").unwrap(),
        ];
        assert!(
            policy
                .check_resolved_addresses("schema.example", &mixed)
                .is_ok()
        );
    }

    #[test]
    fn unsupported_scheme_is_rejected() {
        let policy = SchemaFetchPolicy::default();
        let uri = tombi_uri::Uri::from_str("file:///tmp/schema.json").unwrap();

        assert!(matches!(
            policy.check_http_url(&uri),
            Err(super::PolicyError::Denied { .. })
        ));
    }

    #[test]
    fn url_credentials_are_rejected() {
        let policy = SchemaFetchPolicy::default();
        let uri = tombi_uri::Uri::from_str("https://user:pass@example.com/schema.json").unwrap();

        assert!(matches!(
            policy.check_http_url(&uri),
            Err(super::PolicyError::Denied { .. })
        ));
    }

    #[test]
    fn trusted_hosts_reject_credentials_and_ports() {
        assert!(super::parse_trusted_host("user@schema.example").is_err());
        assert!(super::parse_trusted_host("schema.example:443").is_err());
        assert_eq!(super::parse_trusted_host("[fd00::1]").unwrap(), "fd00::1");
    }
}
