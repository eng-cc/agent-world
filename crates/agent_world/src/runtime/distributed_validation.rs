#[derive(Debug, Clone)]
pub struct HeadValidationResult {
    pub block_hash: String,
    pub snapshot: super::snapshot::Snapshot,
    pub journal: super::snapshot::Journal,
}

include!("../../../agent_world_net/src/head_validation.rs");
