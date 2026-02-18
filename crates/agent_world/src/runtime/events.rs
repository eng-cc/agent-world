//! Action and domain event types.

use crate::geometry::GeoPos;
use crate::models::{BodyKernelView, BodySlotType};
use crate::simulator::ResourceKind;
use agent_world_wasm_abi::{
    FactoryModuleSpec, MaterialStack, ModuleManifest, ProductValidationDecision,
    RecipeExecutionPlan,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

use super::node_points::EpochSettlementReport;
use super::reward_asset::NodeRewardMintRecord;
use super::types::{ActionId, MaterialLedgerId, ProposalId, WorldTime};

fn default_world_material_ledger() -> MaterialLedgerId {
    MaterialLedgerId::world()
}

/// An envelope wrapping an action with its ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionEnvelope {
    pub id: ActionId,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub time: WorldTime,
    pub agent_id: String,
    pub pos: GeoPos,
    pub visibility_range_cm: i64,
    pub visible_agents: Vec<ObservedAgent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedAgent {
    pub agent_id: String,
    pub pos: GeoPos,
    pub distance_cm: i64,
}

/// Actions that can be submitted to the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Action {
    RegisterAgent {
        agent_id: String,
        pos: GeoPos,
    },
    MoveAgent {
        agent_id: String,
        to: GeoPos,
    },
    QueryObservation {
        agent_id: String,
    },
    EmitObservation {
        observation: Observation,
    },
    BodyAction {
        agent_id: String,
        kind: String,
        payload: JsonValue,
    },
    EmitBodyAttributes {
        agent_id: String,
        view: BodyKernelView,
        reason: String,
    },
    ExpandBodyInterface {
        agent_id: String,
        interface_module_item_id: String,
    },
    DeployModuleArtifact {
        publisher_agent_id: String,
        wasm_hash: String,
        wasm_bytes: Vec<u8>,
    },
    InstallModuleFromArtifact {
        installer_agent_id: String,
        manifest: ModuleManifest,
        activate: bool,
    },
    TransferResource {
        from_agent_id: String,
        to_agent_id: String,
        kind: ResourceKind,
        amount: i64,
    },
    RedeemPower {
        node_id: String,
        target_agent_id: String,
        redeem_credits: u64,
        nonce: u64,
    },
    RedeemPowerSigned {
        node_id: String,
        target_agent_id: String,
        redeem_credits: u64,
        nonce: u64,
        signer_node_id: String,
        signature: String,
    },
    ApplyNodePointsSettlementSigned {
        report: EpochSettlementReport,
        signer_node_id: String,
        mint_records: Vec<NodeRewardMintRecord>,
    },
    TransferMaterial {
        requester_agent_id: String,
        from_ledger: MaterialLedgerId,
        to_ledger: MaterialLedgerId,
        kind: String,
        amount: i64,
        distance_km: i64,
    },
    EmitResourceTransfer {
        from_agent_id: String,
        to_agent_id: String,
        kind: ResourceKind,
        amount: i64,
    },
    BuildFactory {
        builder_agent_id: String,
        site_id: String,
        spec: FactoryModuleSpec,
    },
    BuildFactoryWithModule {
        builder_agent_id: String,
        site_id: String,
        module_id: String,
        spec: FactoryModuleSpec,
    },
    ScheduleRecipe {
        requester_agent_id: String,
        factory_id: String,
        recipe_id: String,
        plan: RecipeExecutionPlan,
    },
    ScheduleRecipeWithModule {
        requester_agent_id: String,
        factory_id: String,
        recipe_id: String,
        module_id: String,
        desired_batches: u32,
        deterministic_seed: u64,
    },
    ValidateProduct {
        requester_agent_id: String,
        module_id: String,
        stack: MaterialStack,
        decision: ProductValidationDecision,
    },
    ValidateProductWithModule {
        requester_agent_id: String,
        module_id: String,
        stack: MaterialStack,
        deterministic_seed: u64,
    },
}

/// Domain events that describe state changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DomainEvent {
    AgentRegistered {
        agent_id: String,
        pos: GeoPos,
    },
    AgentMoved {
        agent_id: String,
        from: GeoPos,
        to: GeoPos,
    },
    ActionRejected {
        action_id: ActionId,
        reason: RejectReason,
    },
    Observation {
        observation: Observation,
    },
    BodyAttributesUpdated {
        agent_id: String,
        view: BodyKernelView,
        reason: String,
    },
    BodyAttributesRejected {
        agent_id: String,
        reason: String,
    },
    BodyInterfaceExpanded {
        agent_id: String,
        slot_capacity: u16,
        expansion_level: u16,
        consumed_item_id: String,
        new_slot_id: String,
        slot_type: BodySlotType,
    },
    BodyInterfaceExpandRejected {
        agent_id: String,
        consumed_item_id: String,
        reason: String,
    },
    ModuleArtifactDeployed {
        publisher_agent_id: String,
        wasm_hash: String,
        bytes_len: u64,
    },
    ModuleInstalled {
        installer_agent_id: String,
        module_id: String,
        module_version: String,
        active: bool,
        proposal_id: ProposalId,
        manifest_hash: String,
    },
    ResourceTransferred {
        from_agent_id: String,
        to_agent_id: String,
        kind: ResourceKind,
        amount: i64,
    },
    PowerRedeemed {
        node_id: String,
        target_agent_id: String,
        burned_credits: u64,
        granted_power_units: i64,
        reserve_remaining: i64,
        nonce: u64,
    },
    PowerRedeemRejected {
        node_id: String,
        target_agent_id: String,
        redeem_credits: u64,
        nonce: u64,
        reason: String,
    },
    NodePointsSettlementApplied {
        report: EpochSettlementReport,
        signer_node_id: String,
        settlement_hash: String,
        minted_records: Vec<NodeRewardMintRecord>,
    },
    MaterialTransferred {
        requester_agent_id: String,
        from_ledger: MaterialLedgerId,
        to_ledger: MaterialLedgerId,
        kind: String,
        amount: i64,
        distance_km: i64,
    },
    MaterialTransitStarted {
        job_id: ActionId,
        requester_agent_id: String,
        from_ledger: MaterialLedgerId,
        to_ledger: MaterialLedgerId,
        kind: String,
        amount: i64,
        distance_km: i64,
        loss_bps: i64,
        ready_at: WorldTime,
    },
    MaterialTransitCompleted {
        job_id: ActionId,
        requester_agent_id: String,
        from_ledger: MaterialLedgerId,
        to_ledger: MaterialLedgerId,
        kind: String,
        sent_amount: i64,
        received_amount: i64,
        loss_amount: i64,
        distance_km: i64,
    },
    FactoryBuildStarted {
        job_id: ActionId,
        builder_agent_id: String,
        site_id: String,
        spec: FactoryModuleSpec,
        #[serde(default = "default_world_material_ledger")]
        consume_ledger: MaterialLedgerId,
        ready_at: WorldTime,
    },
    FactoryBuilt {
        job_id: ActionId,
        builder_agent_id: String,
        site_id: String,
        spec: FactoryModuleSpec,
    },
    RecipeStarted {
        job_id: ActionId,
        requester_agent_id: String,
        factory_id: String,
        recipe_id: String,
        accepted_batches: u32,
        consume: Vec<MaterialStack>,
        produce: Vec<MaterialStack>,
        byproducts: Vec<MaterialStack>,
        power_required: i64,
        duration_ticks: u32,
        #[serde(default = "default_world_material_ledger")]
        consume_ledger: MaterialLedgerId,
        #[serde(default = "default_world_material_ledger")]
        output_ledger: MaterialLedgerId,
        ready_at: WorldTime,
    },
    RecipeCompleted {
        job_id: ActionId,
        requester_agent_id: String,
        factory_id: String,
        recipe_id: String,
        accepted_batches: u32,
        produce: Vec<MaterialStack>,
        byproducts: Vec<MaterialStack>,
        #[serde(default = "default_world_material_ledger")]
        output_ledger: MaterialLedgerId,
    },
    ProductValidated {
        requester_agent_id: String,
        module_id: String,
        stack: MaterialStack,
        stack_limit: u32,
        tradable: bool,
        quality_levels: Vec<String>,
        notes: Vec<String>,
    },
}

impl DomainEvent {
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            DomainEvent::AgentRegistered { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::AgentMoved { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::Observation { observation } => Some(observation.agent_id.as_str()),
            DomainEvent::BodyAttributesUpdated { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::BodyAttributesRejected { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::BodyInterfaceExpanded { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::BodyInterfaceExpandRejected { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::ModuleArtifactDeployed {
                publisher_agent_id, ..
            } => Some(publisher_agent_id.as_str()),
            DomainEvent::ModuleInstalled {
                installer_agent_id, ..
            } => Some(installer_agent_id.as_str()),
            DomainEvent::ActionRejected { .. } => None,
            DomainEvent::ResourceTransferred { from_agent_id, .. } => Some(from_agent_id.as_str()),
            DomainEvent::PowerRedeemed {
                target_agent_id, ..
            } => Some(target_agent_id.as_str()),
            DomainEvent::PowerRedeemRejected {
                target_agent_id, ..
            } => Some(target_agent_id.as_str()),
            DomainEvent::NodePointsSettlementApplied { .. } => None,
            DomainEvent::MaterialTransferred {
                requester_agent_id, ..
            } => Some(requester_agent_id.as_str()),
            DomainEvent::MaterialTransitStarted {
                requester_agent_id, ..
            } => Some(requester_agent_id.as_str()),
            DomainEvent::MaterialTransitCompleted {
                requester_agent_id, ..
            } => Some(requester_agent_id.as_str()),
            DomainEvent::FactoryBuildStarted {
                builder_agent_id, ..
            } => Some(builder_agent_id.as_str()),
            DomainEvent::FactoryBuilt {
                builder_agent_id, ..
            } => Some(builder_agent_id.as_str()),
            DomainEvent::RecipeStarted {
                requester_agent_id, ..
            } => Some(requester_agent_id.as_str()),
            DomainEvent::RecipeCompleted {
                requester_agent_id, ..
            } => Some(requester_agent_id.as_str()),
            DomainEvent::ProductValidated {
                requester_agent_id, ..
            } => Some(requester_agent_id.as_str()),
        }
    }
}

/// Reasons why an action was rejected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RejectReason {
    AgentAlreadyExists {
        agent_id: String,
    },
    AgentNotFound {
        agent_id: String,
    },
    AgentsNotCoLocated {
        agent_id: String,
        other_agent_id: String,
    },
    InvalidAmount {
        amount: i64,
    },
    InsufficientResource {
        agent_id: String,
        kind: ResourceKind,
        requested: i64,
        available: i64,
    },
    InsufficientResources {
        deficits: BTreeMap<ResourceKind, i64>,
    },
    InsufficientMaterial {
        material_kind: String,
        requested: i64,
        available: i64,
    },
    MaterialTransferDistanceExceeded {
        distance_km: i64,
        max_distance_km: i64,
    },
    MaterialTransitCapacityExceeded {
        in_flight: usize,
        max_in_flight: usize,
    },
    FactoryNotFound {
        factory_id: String,
    },
    FactoryBusy {
        factory_id: String,
        active_jobs: usize,
        recipe_slots: u16,
    },
    RuleDenied {
        notes: Vec<String>,
    },
}

/// The cause of an event, for audit purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CausedBy {
    Action(ActionId),
    Effect { intent_id: String },
}
