#[derive(Debug, Clone, thiserror::Error)]
pub enum FetchError {
    #[error("destination host is not trusted")]
    HostNotTrusted,
    #[error("{reason}")]
    PolicyDenied { reason: String },
    #[error("{reason}")]
    FetchFailed { reason: String },
    #[error("unexpected status: {status}")]
    StatusNotOk { status: u16 },
    #[error("GitHub authentication failed: {reason}")]
    AuthenticationFailed { reason: String },
    #[error("failed to read body: {reason}")]
    BodyReadFailed { reason: String },
}

impl FetchError {
    /// Whether the schema access policy refused the request.
    ///
    /// Policy refusals are terminal: they must not fall back to cached content.
    pub fn is_policy_denied(&self) -> bool {
        matches!(self, Self::PolicyDenied { .. } | Self::HostNotTrusted)
    }
}

impl From<crate::schema_fetch_policy::PolicyError> for FetchError {
    fn from(error: crate::schema_fetch_policy::PolicyError) -> Self {
        use crate::schema_fetch_policy::PolicyError;
        match error {
            PolicyError::HostNotTrusted => Self::HostNotTrusted,
            error @ PolicyError::Denied { .. } => Self::PolicyDenied {
                reason: error.to_string(),
            },
        }
    }
}
