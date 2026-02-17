use crate::geometry::GeoPos;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::super::agent::{LlmEffectIntentTrace, LlmEffectReceiptTrace};
use super::super::chunking::ChunkCoord;
use super::super::module_visual::ModuleVisualEntity;
use super::super::power::PowerEvent;
use super::super::types::{
    Action, ActionId, AgentId, ChunkResourceBudget, FacilityId, FragmentElementKind, LocationId,
    LocationProfile, ResourceKind, ResourceOwner, ResourceStock, WorldEventId, WorldTime,
};
use super::super::world_model::{AgentPromptProfile, Location};

// ============================================================================
// Observation Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub time: WorldTime,
    pub agent_id: AgentId,
    pub pos: GeoPos,
    pub self_resources: ResourceStock,
    pub visibility_range_cm: i64,
    pub visible_agents: Vec<ObservedAgent>,
    pub visible_locations: Vec<ObservedLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedAgent {
    pub agent_id: AgentId,
    pub location_id: LocationId,
    pub pos: GeoPos,
    pub distance_cm: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedLocation {
    pub location_id: LocationId,
    pub name: String,
    pub pos: GeoPos,
    pub profile: LocationProfile,
    pub distance_cm: i64,
}

// ============================================================================
// Event Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: WorldEventId,
    pub time: WorldTime,
    pub kind: WorldEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentReplenishedEntry {
    pub coord: ChunkCoord,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WorldEventKind {
    LocationRegistered {
        location_id: LocationId,
        name: String,
        pos: GeoPos,
        profile: LocationProfile,
    },
    AgentRegistered {
        agent_id: AgentId,
        location_id: LocationId,
        pos: GeoPos,
    },
    AgentMoved {
        agent_id: AgentId,
        from: LocationId,
        to: LocationId,
        distance_cm: i64,
        electricity_cost: i64,
    },
    ResourceTransferred {
        from: ResourceOwner,
        to: ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    },
    RadiationHarvested {
        agent_id: AgentId,
        location_id: LocationId,
        amount: i64,
        available: i64,
    },
    CompoundMined {
        owner: ResourceOwner,
        location_id: LocationId,
        compound_mass_g: i64,
        electricity_cost: i64,
        extracted_elements: BTreeMap<FragmentElementKind, i64>,
    },
    CompoundRefined {
        owner: ResourceOwner,
        compound_mass_g: i64,
        electricity_cost: i64,
        hardware_output: i64,
    },
    FactoryBuilt {
        owner: ResourceOwner,
        location_id: LocationId,
        factory_id: FacilityId,
        factory_kind: String,
        electricity_cost: i64,
        hardware_cost: i64,
    },
    RecipeScheduled {
        owner: ResourceOwner,
        factory_id: FacilityId,
        recipe_id: String,
        batches: i64,
        electricity_cost: i64,
        hardware_cost: i64,
        data_output: i64,
        finished_product_id: String,
        finished_product_units: i64,
    },
    ModuleVisualEntityUpserted {
        entity: ModuleVisualEntity,
    },
    ModuleVisualEntityRemoved {
        entity_id: String,
    },
    ChunkGenerated {
        coord: ChunkCoord,
        seed: u64,
        fragment_count: u32,
        block_count: u32,
        chunk_budget: ChunkResourceBudget,
        cause: ChunkGenerationCause,
    },
    FragmentsReplenished {
        entries: Vec<FragmentReplenishedEntry>,
    },
    AgentPromptUpdated {
        profile: AgentPromptProfile,
        operation: PromptUpdateOperation,
        applied_fields: Vec<String>,
        digest: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rolled_back_to_version: Option<u64>,
    },
    ActionRejected {
        reason: RejectReason,
    },
    LlmEffectQueued {
        agent_id: AgentId,
        intent: LlmEffectIntentTrace,
    },
    LlmReceiptAppended {
        agent_id: AgentId,
        receipt: LlmEffectReceiptTrace,
    },
    // Power system events
    Power(PowerEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptUpdateOperation {
    Apply,
    Rollback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkGenerationCause {
    Init,
    Observe,
    Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RejectReason {
    AgentAlreadyExists {
        agent_id: AgentId,
    },
    AgentNotFound {
        agent_id: AgentId,
    },
    LocationAlreadyExists {
        location_id: LocationId,
    },
    LocationNotFound {
        location_id: LocationId,
    },
    FacilityAlreadyExists {
        facility_id: FacilityId,
    },
    FacilityNotFound {
        facility_id: FacilityId,
    },
    AgentAlreadyAtLocation {
        agent_id: AgentId,
        location_id: LocationId,
    },
    MoveDistanceExceeded {
        distance_cm: i64,
        max_distance_cm: i64,
    },
    MoveSpeedExceeded {
        required_speed_cm_per_s: i64,
        max_speed_cm_per_s: i64,
        time_step_s: i64,
    },
    InvalidAmount {
        amount: i64,
    },
    InsufficientResource {
        owner: ResourceOwner,
        kind: ResourceKind,
        requested: i64,
        available: i64,
    },
    LocationTransferNotAllowed {
        from: LocationId,
        to: LocationId,
    },
    AgentNotAtLocation {
        agent_id: AgentId,
        location_id: LocationId,
    },
    AgentsNotCoLocated {
        agent_id: AgentId,
        other_agent_id: AgentId,
    },
    AgentShutdown {
        agent_id: AgentId,
    },
    PositionOutOfBounds {
        pos: GeoPos,
    },
    ChunkGenerationFailed {
        x: i32,
        y: i32,
        z: i32,
    },
    RadiationUnavailable {
        location_id: LocationId,
    },
    ThermalOverload {
        heat: i64,
        capacity: i64,
    },
    PowerTransferDistanceExceeded {
        distance_km: i64,
        max_distance_km: i64,
    },
    PowerTransferLossExceedsAmount {
        amount: i64,
        loss: i64,
    },
    RuleDenied {
        #[serde(default)]
        notes: Vec<String>,
    },
}

// ============================================================================
// Rule Decision Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct KernelRuleCost {
    pub entries: BTreeMap<ResourceKind, i64>,
}

impl KernelRuleCost {
    pub fn add_assign(&mut self, other: &KernelRuleCost) {
        for (kind, delta) in &other.entries {
            *self.entries.entry(*kind).or_insert(0) += *delta;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelRuleVerdict {
    Allow,
    Deny,
    Modify,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KernelRuleDecision {
    pub action_id: ActionId,
    pub verdict: KernelRuleVerdict,
    #[serde(default)]
    pub override_action: Option<Action>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub cost: KernelRuleCost,
}

impl KernelRuleDecision {
    pub fn allow(action_id: ActionId) -> Self {
        Self {
            action_id,
            verdict: KernelRuleVerdict::Allow,
            override_action: None,
            notes: Vec::new(),
            cost: KernelRuleCost::default(),
        }
    }

    pub fn deny(action_id: ActionId, notes: Vec<String>) -> Self {
        Self {
            action_id,
            verdict: KernelRuleVerdict::Deny,
            override_action: None,
            notes,
            cost: KernelRuleCost::default(),
        }
    }

    pub fn modify(action_id: ActionId, override_action: Action) -> Self {
        Self {
            action_id,
            verdict: KernelRuleVerdict::Modify,
            override_action: Some(override_action),
            notes: Vec::new(),
            cost: KernelRuleCost::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct KernelRuleModuleContext {
    pub time: WorldTime,
    #[serde(default)]
    pub location_ids: Vec<LocationId>,
    #[serde(default)]
    pub agent_ids: Vec<AgentId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KernelRuleModuleInput {
    pub action_id: ActionId,
    pub action: Action,
    pub context: KernelRuleModuleContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KernelRuleModuleOutput {
    pub decision: KernelRuleDecision,
}

impl KernelRuleModuleOutput {
    pub fn from_decision(decision: KernelRuleDecision) -> Self {
        Self { decision }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelRuleDecisionMergeError {
    ActionIdMismatch { expected: ActionId, got: ActionId },
    MissingOverride { action_id: ActionId },
    ConflictingOverride { action_id: ActionId },
}

impl std::fmt::Display for KernelRuleDecisionMergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ActionIdMismatch { expected, got } => {
                write!(
                    f,
                    "rule decision action_id mismatch expected {expected} got {got}"
                )
            }
            Self::MissingOverride { action_id } => {
                write!(f, "rule decision missing override for action {action_id}")
            }
            Self::ConflictingOverride { action_id } => {
                write!(
                    f,
                    "rule decision conflicting override for action {action_id}"
                )
            }
        }
    }
}

pub fn merge_kernel_rule_decisions<I>(
    action_id: ActionId,
    decisions: I,
) -> Result<KernelRuleDecision, KernelRuleDecisionMergeError>
where
    I: IntoIterator<Item = KernelRuleDecision>,
{
    let mut merged = KernelRuleDecision::allow(action_id);
    let mut has_deny = false;

    for decision in decisions {
        if decision.action_id != action_id {
            return Err(KernelRuleDecisionMergeError::ActionIdMismatch {
                expected: action_id,
                got: decision.action_id,
            });
        }

        merged.cost.add_assign(&decision.cost);
        merged.notes.extend(decision.notes);

        match decision.verdict {
            KernelRuleVerdict::Deny => {
                merged.verdict = KernelRuleVerdict::Deny;
                has_deny = true;
            }
            KernelRuleVerdict::Modify => {
                if has_deny {
                    continue;
                }
                let Some(action) = decision.override_action else {
                    return Err(KernelRuleDecisionMergeError::MissingOverride { action_id });
                };
                match &merged.override_action {
                    Some(existing) if existing != &action => {
                        return Err(KernelRuleDecisionMergeError::ConflictingOverride {
                            action_id,
                        });
                    }
                    Some(_) => {}
                    None => merged.override_action = Some(action),
                }
                merged.verdict = KernelRuleVerdict::Modify;
            }
            KernelRuleVerdict::Allow => {}
        }
    }

    Ok(merged)
}
