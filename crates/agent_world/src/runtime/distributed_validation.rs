#[derive(Debug, Clone)]
pub struct HeadValidationResult {
    pub block_hash: String,
    pub snapshot: super::snapshot::Snapshot,
    pub journal: super::snapshot::Journal,
}

mod blob_store {
    pub(super) use super::super::blob_store::*;
}

mod distributed {
    pub(super) use super::super::distributed::*;
}

#[cfg(test)]
mod distributed_storage {
    pub(super) use super::super::distributed_storage::*;
}

mod distributed_validation {
    pub(super) use super::HeadValidationResult;
}

mod error {
    pub(super) use super::super::error::WorldError;
}

mod events {
    pub(super) use super::super::events::*;
}

mod segmenter {
    pub(super) use super::super::segmenter::*;
}

mod snapshot {
    pub(super) use super::super::snapshot::*;
}

mod types {
    pub(super) use super::super::types::*;
}

mod util {
    pub(super) use super::super::util::*;
}

mod world {
    pub(super) use super::super::world::World;
}

mod world_event {
    pub(super) use super::super::world_event::*;
}

#[path = "../../../agent_world_net/src/head_validation.rs"]
#[allow(dead_code, unused_imports)]
mod shared;

pub use shared::{assemble_journal, assemble_snapshot, validate_head_update};
