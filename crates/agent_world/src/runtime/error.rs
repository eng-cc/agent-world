//! Error types for the runtime module.

use std::io;

use super::sandbox::ModuleCallErrorCode;
use super::types::ProposalId;

/// Errors that can occur in world operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldError {
    JournalMismatch,
    AgentNotFound {
        agent_id: String,
    },
    ResourceBalanceInvalid {
        reason: String,
    },
    CapabilityMissing {
        cap_ref: String,
    },
    CapabilityExpired {
        cap_ref: String,
    },
    CapabilityNotAllowed {
        cap_ref: String,
        kind: String,
    },
    PolicyDenied {
        intent_id: String,
        reason: String,
    },
    ReceiptUnknownIntent {
        intent_id: String,
    },
    ReceiptSignatureInvalid {
        intent_id: String,
    },
    ProposalNotFound {
        proposal_id: ProposalId,
    },
    ProposalInvalidState {
        proposal_id: ProposalId,
        expected: String,
        found: String,
    },
    PatchBaseMismatch {
        expected: String,
        found: String,
    },
    PatchInvalidPath {
        path: String,
    },
    PatchNonObject {
        path: String,
    },
    ModuleChangeInvalid {
        reason: String,
    },
    ModuleCallFailed {
        module_id: String,
        trace_id: String,
        code: ModuleCallErrorCode,
        detail: String,
    },
    ModuleStoreVersionMismatch {
        expected: u64,
        found: u64,
    },
    ModuleStoreArtifactMissing {
        wasm_hash: String,
    },
    ModuleStoreManifestMismatch {
        wasm_hash: String,
    },
    BlobNotFound {
        content_hash: String,
    },
    BlobHashMismatch {
        expected: String,
        actual: String,
    },
    BlobHashInvalid {
        content_hash: String,
    },
    NetworkProtocolUnavailable {
        protocol: String,
    },
    NetworkRequestFailed {
        code: super::distributed::DistributedErrorCode,
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

impl From<serde_json::Error> for WorldError {
    fn from(error: serde_json::Error) -> Self {
        WorldError::Serde(error.to_string())
    }
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
