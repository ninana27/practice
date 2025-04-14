use chacha20poly1305::XChaCha20Poly1305;
use ed25519_dalek::ed25519;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("{0}")]
    Internal(String),

    #[error("{0}")]
    InvalidArgument(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(err: ed25519::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Internal(err.to_string())
    }
}

impl From<chacha20poly1305::Error> for Error {
    fn from(err: chacha20poly1305::Error) -> Self {
        Error::Internal(err.to_string())
    }
}
