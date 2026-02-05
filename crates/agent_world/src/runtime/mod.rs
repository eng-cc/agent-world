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
pub mod distributed;
mod distributed_client;
mod distributed_net;
mod effect;
mod error;
mod events;
mod governance;
mod manifest;
mod modules;
mod module_store;
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
    ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits, ModuleManifest, ModuleRegistry,
    ModuleRecord, ModuleRole, ModuleSubscription, ModuleSubscriptionStage, ModuleUpgrade,
};

// Blob store
pub use blob_store::{blake3_hex, BlobStore, LocalCasStore};

// Module store
pub use module_store::ModuleStore;

// Distributed network adapter
pub use distributed_net::{
    DistributedNetwork, InMemoryNetwork, NetworkMessage, NetworkRequest, NetworkResponse,
    NetworkSubscription,
};

// Distributed client
pub use distributed_client::DistributedClient;

// Segmenter
pub use segmenter::{segment_journal, segment_snapshot, JournalSegmentRef, SegmentConfig};

// Policy
pub use policy::{PolicyDecision, PolicyDecisionRecord, PolicyRule, PolicySet, PolicyWhen};

// Rules
pub use rules::{merge_rule_decisions, ResourceDelta, RuleDecision, RuleDecisionMergeError, RuleVerdict};

// Signer
pub use signer::ReceiptSigner;

// Sandbox
pub use sandbox::{
    FixedSandbox, ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallOrigin,
    ModuleCallRequest, ModuleContext, ModuleEmit, ModuleEmitEvent, ModuleEffectIntent,
    ModuleOutput, ModuleSandbox, ModuleStateUpdate, WasmEngineKind, WasmExecutor, WasmExecutorConfig,
};

// Snapshot
pub use snapshot::{
    Journal, RollbackEvent, Snapshot, SnapshotCatalog, SnapshotMeta, SnapshotRecord,
    SnapshotRetentionPolicy,
};

// State
pub use state::WorldState;

// World
pub use world::World;

// World event
pub use world_event::{WorldEvent, WorldEventBody};
