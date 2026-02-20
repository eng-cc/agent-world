//! Module types and registry for WASM runtime integration.

pub use agent_world_wasm_abi::{
    EconomyModuleKind, FactoryBuildDecision, FactoryBuildRequest, FactoryModuleApi,
    FactoryModuleSpec, GameplayContract, GameplayModuleKind, MaterialStack, ModuleAbiContract,
    ModuleActivation, ModuleArtifact, ModuleArtifactIdentity, ModuleCache, ModuleChangeSet,
    ModuleDeactivation, ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits, ModuleManifest,
    ModuleRecord, ModuleRegistry, ModuleRole, ModuleSubscription, ModuleSubscriptionStage,
    ModuleUpgrade, ProductModuleApi, ProductModuleSpec, ProductValidationDecision,
    ProductValidationRequest, RecipeExecutionPlan, RecipeExecutionRequest, RecipeModuleApi,
    RecipeModuleSpec,
};
