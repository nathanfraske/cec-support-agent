use thiserror::Error;

/// Errors raised while talking to an inference endpoint.
#[derive(Debug, Error)]
pub enum InferenceError {
    /// Transport-level failure (connection, timeout, TLS, decode).
    #[error("inference transport error: {0}")]
    Transport(#[from] reqwest::Error),
    /// The endpoint returned a non-success HTTP status.
    #[error("inference endpoint returned status {status}: {body}")]
    Status { status: u16, body: String },
    /// The endpoint returned a body with no choices.
    #[error("inference endpoint returned no choices")]
    EmptyResponse,
    /// A payload failed to (de)serialize.
    #[error("inference payload (de)serialization failed: {0}")]
    Serde(#[from] serde_json::Error),
}
