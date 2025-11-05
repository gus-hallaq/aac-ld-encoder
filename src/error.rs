// error.rs - Error handling types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AacLdError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Buffer size mismatch: expected {expected}, got {actual}")]
    BufferSizeMismatch { expected: usize, actual: usize },
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),
    #[error("Bitstream error: {0}")]
    BitstreamError(String),
}

pub type Result<T> = std::result::Result<T, AacLdError>;