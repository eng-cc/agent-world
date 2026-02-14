//! Module types and registry for WASM runtime integration.

pub use agent_world_wasm_abi::{
    ModuleActivation, ModuleArtifact, ModuleCache, ModuleChangeSet, ModuleDeactivation,
    ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits, ModuleManifest, ModuleRecord,
    ModuleRegistry, ModuleRole, ModuleSubscription, ModuleSubscriptionStage, ModuleUpgrade,
};
