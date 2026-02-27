//! Module types and registry for WASM runtime integration.

pub use agent_world_wasm_abi::{
    EconomyModuleKind, FactoryBuildDecision, FactoryBuildRequest, FactoryModuleApi,
    FactoryModuleSpec, GameplayContract, GameplayModuleKind, MaterialDefaultPriority,
    MaterialProfileV1, MaterialStack, MaterialTransportLossClass, ModuleAbiContract,
    ModuleActivation, ModuleArtifact, ModuleArtifactIdentity, ModuleCache, ModuleChangeSet,
    ModuleDeactivation, ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits, ModuleManifest,
    ModuleRecord, ModuleRegistry, ModuleRole, ModuleSubscription, ModuleSubscriptionStage,
    ModuleUpgrade, ProductModuleApi, ProductModuleSpec, ProductProfileV1,
    ProductValidationDecision, ProductValidationRequest, RecipeExecutionPlan,
    RecipeExecutionRequest, RecipeModuleApi, RecipeModuleSpec, RecipeProfileV1,
};
