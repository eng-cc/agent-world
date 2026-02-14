//! Runtime module - the core world execution engine.
//!
//! This module contains the World struct and all supporting types for:
//! - World state management
//! - Event processing and journaling
//! - Snapshot persistence and recovery
//! - Effect system with capabilities and policies
//! - Governance and manifest management
//! - Agent scheduling

mod agent_cell;
mod audit;
mod blob_store;
mod effect;
mod error;
mod events;
mod governance;
mod m1_builtin_wasm_artifact;
mod manifest;
mod module_store;
mod modules;
mod policy;
mod rules;
mod sandbox;
mod segmenter;
mod signer;
mod snapshot;
mod state;
mod types;
mod util;
mod world;
mod world_event;

#[cfg(test)]
mod tests;

// Re-export all public types

// Types
pub use types::{ActionId, IntentSeq, PatchPath, ProposalId, WorldEventId, WorldTime};

// Agent cell
pub use agent_cell::AgentCell;

// Audit
pub use audit::{AuditCausedBy, AuditEventKind, AuditFilter};

// Effect system
pub use effect::{
    CapabilityGrant, EffectIntent, EffectOrigin, EffectReceipt, OriginKind, ReceiptSignature,
    SignatureAlgorithm,
};

// Error
pub use error::WorldError;

// Events
pub use events::{Action, ActionEnvelope, CausedBy, DomainEvent, RejectReason};

// Governance
pub use governance::{AgentSchedule, GovernanceEvent, Proposal, ProposalDecision, ProposalStatus};

// Manifest
pub use manifest::{
    apply_manifest_patch, diff_manifest, merge_manifest_patches,
    merge_manifest_patches_with_conflicts, ConflictKind, Manifest, ManifestPatch, ManifestPatchOp,
    ManifestUpdate, PatchConflict, PatchMergeResult, PatchOpKind, PatchOpSummary,
};

// Modules
pub use modules::{
    ModuleActivation, ModuleArtifact, ModuleCache, ModuleChangeSet, ModuleDeactivation,
    ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits, ModuleManifest, ModuleRecord,
    ModuleRegistry, ModuleRole, ModuleSubscription, ModuleSubscriptionStage, ModuleUpgrade,
};

// Blob store
pub use blob_store::{blake3_hex, BlobStore, LocalCasStore};

pub(crate) use m1_builtin_wasm_artifact::m1_builtin_wasm_module_artifact_bytes;
#[cfg(all(test, feature = "wasmtime"))]
pub(crate) use m1_builtin_wasm_artifact::{
    m1_builtin_module_ids_manifest, register_m1_builtin_wasm_module_artifact,
};

// Built-in module constants
pub use agent_world_builtin_wasm::{
    M1_AGENT_DEFAULT_MODULE_VERSION, M1_BODY_ACTION_COST_ELECTRICITY, M1_BODY_MODULE_ID,
    M1_MEMORY_MAX_ENTRIES, M1_MEMORY_MODULE_ID, M1_MOBILITY_MODULE_ID, M1_MOVE_RULE_MODULE_ID,
    M1_POWER_HARVEST_BASE_PER_TICK, M1_POWER_HARVEST_DISTANCE_BONUS_CAP,
    M1_POWER_HARVEST_DISTANCE_STEP_CM, M1_POWER_MODULE_VERSION, M1_POWER_STORAGE_CAPACITY,
    M1_POWER_STORAGE_INITIAL_LEVEL, M1_POWER_STORAGE_MOVE_COST_PER_KM,
    M1_RADIATION_POWER_MODULE_ID, M1_SENSOR_MODULE_ID, M1_STORAGE_CARGO_MODULE_ID,
    M1_STORAGE_POWER_MODULE_ID, M1_TRANSFER_RULE_MODULE_ID, M1_VISIBILITY_RULE_MODULE_ID,
};

// Module store
pub use module_store::ModuleStore;

// Segmenter
pub use segmenter::{segment_journal, segment_snapshot, JournalSegmentRef, SegmentConfig};

// Policy
pub use policy::{PolicyDecision, PolicyDecisionRecord, PolicyRule, PolicySet, PolicyWhen};

// Rules
pub use rules::{
    merge_rule_decisions, ActionOverrideRecord, ResourceBalanceError, ResourceDelta, RuleDecision,
    RuleDecisionMergeError, RuleDecisionRecord, RuleVerdict,
};

// Signer
pub use signer::ReceiptSigner;

// Sandbox
pub use sandbox::{
    FixedSandbox, ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallOrigin,
    ModuleCallRequest, ModuleContext, ModuleEffectIntent, ModuleEmit, ModuleEmitEvent,
    ModuleOutput, ModuleSandbox, ModuleStateUpdate, WasmEngineKind, WasmExecutor,
    WasmExecutorConfig,
};

// Snapshot
pub use snapshot::{
    Journal, RollbackEvent, Snapshot, SnapshotCatalog, SnapshotMeta, SnapshotRecord,
    SnapshotRetentionPolicy,
};

// State
pub use state::WorldState;

// World
pub use world::{M1ScenarioBootstrapConfig, World};

// World event
pub use world_event::{WorldEvent, WorldEventBody};
