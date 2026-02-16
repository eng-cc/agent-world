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
mod builtin_wasm_materializer;
mod effect;
mod error;
mod events;
mod governance;
mod m1_builtin_wasm_artifact;
mod m4_builtin_wasm_artifact;
mod manifest;
mod module_store;
mod modules;
mod node_points;
mod policy;
mod rules;
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
pub use types::{
    ActionId, IntentSeq, MaterialLedgerId, PatchPath, ProposalId, WorldEventId, WorldTime,
};

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
    EconomyModuleKind, FactoryBuildDecision, FactoryBuildRequest, FactoryModuleApi,
    FactoryModuleSpec, MaterialStack, ModuleActivation, ModuleArtifact, ModuleArtifactIdentity,
    ModuleCache, ModuleChangeSet, ModuleDeactivation, ModuleEvent, ModuleEventKind, ModuleKind,
    ModuleLimits, ModuleManifest, ModuleRecord, ModuleRegistry, ModuleRole, ModuleSubscription,
    ModuleSubscriptionStage, ModuleUpgrade, ProductModuleApi, ProductModuleSpec,
    ProductValidationDecision, ProductValidationRequest, RecipeExecutionPlan,
    RecipeExecutionRequest, RecipeModuleApi, RecipeModuleSpec,
};

// Node points
pub use node_points::{
    EpochSettlementReport, NodeContributionSample, NodePointsConfig, NodePointsLedger,
    NodeSettlement,
};

// Blob store
pub use blob_store::{blake3_hex, BlobStore, HashAlgorithm, LocalCasStore};
pub(crate) use builtin_wasm_materializer::load_builtin_wasm_with_fetch_fallback;

pub(crate) use m1_builtin_wasm_artifact::m1_builtin_wasm_module_artifact_bytes;
#[cfg(all(test, feature = "wasmtime"))]
pub(crate) use m1_builtin_wasm_artifact::{
    m1_builtin_module_ids_manifest, register_m1_builtin_wasm_module_artifact,
};
#[cfg(all(test, feature = "wasmtime"))]
pub(crate) use m4_builtin_wasm_artifact::m4_builtin_module_ids_manifest;
pub(crate) use m4_builtin_wasm_artifact::m4_builtin_wasm_module_artifact_bytes;

// Built-in module constants
pub use agent_world_builtin_wasm::{
    M1_AGENT_DEFAULT_MODULE_VERSION, M1_BODY_ACTION_COST_ELECTRICITY, M1_BODY_MODULE_ID,
    M1_MEMORY_MAX_ENTRIES, M1_MEMORY_MODULE_ID, M1_MOBILITY_MODULE_ID, M1_MOVE_RULE_MODULE_ID,
    M1_POWER_HARVEST_BASE_PER_TICK, M1_POWER_HARVEST_DISTANCE_BONUS_CAP,
    M1_POWER_HARVEST_DISTANCE_STEP_CM, M1_POWER_MODULE_VERSION, M1_POWER_STORAGE_CAPACITY,
    M1_POWER_STORAGE_INITIAL_LEVEL, M1_POWER_STORAGE_MOVE_COST_PER_KM,
    M1_RADIATION_POWER_MODULE_ID, M1_SENSOR_MODULE_ID, M1_STORAGE_CARGO_MODULE_ID,
    M1_STORAGE_POWER_MODULE_ID, M1_TRANSFER_RULE_MODULE_ID, M1_VISIBILITY_RULE_MODULE_ID,
    M4_ECONOMY_MODULE_VERSION, M4_FACTORY_ASSEMBLER_MODULE_ID, M4_FACTORY_MINER_MODULE_ID,
    M4_FACTORY_SMELTER_MODULE_ID, M4_PRODUCT_CONTROL_CHIP_MODULE_ID,
    M4_PRODUCT_IRON_INGOT_MODULE_ID, M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID,
    M4_PRODUCT_MOTOR_MODULE_ID, M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID,
    M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID, M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID,
    M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID, M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID,
    M4_RECIPE_SMELT_IRON_MODULE_ID,
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

// Snapshot
pub use snapshot::{
    Journal, RollbackEvent, Snapshot, SnapshotCatalog, SnapshotMeta, SnapshotRecord,
    SnapshotRetentionPolicy,
};

// State
pub use state::{
    FactoryBuildJobState, FactoryState, MaterialTransitJobState, RecipeJobState, WorldState,
};

// World
pub use world::{M1ScenarioBootstrapConfig, World};

// World event
pub use world_event::{WorldEvent, WorldEventBody};
