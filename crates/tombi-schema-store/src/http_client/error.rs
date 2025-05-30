#[derive(Debug, Clone, thiserror::Error)]
pub enum FetchError {
    #[error("{reason}")]
    FetchFailed { reason: String },
    #[error("unexpected status: {status}")]
    StatusNotOk { status: u16 },
    #[error("failed to read body: {reason}")]
    BodyReadFailed { reason: String },
}
