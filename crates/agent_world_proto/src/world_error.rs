use std::io;

use crate::distributed::DistributedErrorCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldError {
    NetworkProtocolUnavailable {
        protocol: String,
    },
    NetworkRequestFailed {
        code: DistributedErrorCode,
        message: String,
        retryable: bool,
    },
    DistributedValidationFailed {
        reason: String,
    },
    SignatureKeyInvalid,
    Io(String),
    Serde(String),
}

impl From<serde_cbor::Error> for WorldError {
    fn from(error: serde_cbor::Error) -> Self {
        WorldError::Serde(error.to_string())
    }
}

impl From<io::Error> for WorldError {
    fn from(error: io::Error) -> Self {
        WorldError::Io(error.to_string())
    }
}
