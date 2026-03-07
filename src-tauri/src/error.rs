use serde::Serialize;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum AppError {
    #[error("OpenClaw unreachable at {0}. Is it running?")]
    ConnectionFailed(String),

    #[error("Agent '{0}' not found")]
    AgentNotFound(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Not connected. Call connect() first.")]
    NotConnected,
}

