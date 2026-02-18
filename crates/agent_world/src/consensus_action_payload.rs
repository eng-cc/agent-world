use crate::runtime;
use crate::simulator::{Action as SimulatorAction, ActionSubmitter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsensusActionPayloadEnvelope {
    pub version: u8,
    pub body: ConsensusActionPayloadBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ConsensusActionPayloadBody {
    RuntimeAction {
        action: runtime::Action,
    },
    SimulatorAction {
        action: SimulatorAction,
        #[serde(default)]
        submitter: ActionSubmitter,
    },
}

impl ConsensusActionPayloadEnvelope {
    pub fn from_runtime_action(action: runtime::Action) -> Self {
        Self {
            version: 1,
            body: ConsensusActionPayloadBody::RuntimeAction { action },
        }
    }

    pub fn from_simulator_action(action: SimulatorAction, submitter: ActionSubmitter) -> Self {
        Self {
            version: 1,
            body: ConsensusActionPayloadBody::SimulatorAction { action, submitter },
        }
    }
}

pub fn encode_consensus_action_payload(
    envelope: &ConsensusActionPayloadEnvelope,
) -> Result<Vec<u8>, String> {
    serde_cbor::to_vec(envelope)
        .map_err(|err| format!("encode consensus action payload envelope failed: {err}"))
}

pub fn decode_consensus_action_payload(
    payload_cbor: &[u8],
) -> Result<ConsensusActionPayloadBody, String> {
    match serde_cbor::from_slice::<ConsensusActionPayloadEnvelope>(payload_cbor) {
        Ok(envelope) => {
            if envelope.version != 1 {
                return Err(format!(
                    "unsupported consensus payload envelope version {}",
                    envelope.version
                ));
            }
            Ok(envelope.body)
        }
        Err(envelope_err) => match serde_cbor::from_slice::<runtime::Action>(payload_cbor) {
            Ok(action) => Ok(ConsensusActionPayloadBody::RuntimeAction { action }),
            Err(runtime_err) => Err(format!(
                "decode consensus payload envelope failed ({envelope_err}); runtime fallback failed ({runtime_err})"
            )),
        },
    }
}
