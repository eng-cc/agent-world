use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::geometry::GeoPos;
use crate::simulator::CM_PER_KM;

use super::super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallRequest, ModuleOutput,
};
use super::super::util::to_canonical_cbor;
use super::super::world_event::{WorldEvent, WorldEventBody};

mod power_radiation_module;
mod power_storage_module;

pub const M1_RADIATION_POWER_MODULE_ID: &str = "m1.power.radiation_harvest";
pub const M1_STORAGE_POWER_MODULE_ID: &str = "m1.power.storage";
pub const M1_POWER_MODULE_VERSION: &str = "0.1.0";
pub const M1_POWER_STORAGE_CAPACITY: i64 = 12;
pub const M1_POWER_STORAGE_INITIAL_LEVEL: i64 = 6;
pub const M1_POWER_STORAGE_MOVE_COST_PER_KM: i64 = 3;
pub const M1_POWER_HARVEST_BASE_PER_TICK: i64 = 1;
pub const M1_POWER_HARVEST_DISTANCE_STEP_CM: i64 = 800_000;
pub const M1_POWER_HARVEST_DISTANCE_BONUS_CAP: i64 = 1;

pub use power_radiation_module::M1RadiationPowerModule;
pub use power_storage_module::M1StoragePowerModule;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentPowerState {
    pos: GeoPos,
    level: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PowerState {
    agents: BTreeMap<String, AgentPowerState>,
}

fn update_power_state_positions(state: &mut PowerState, event: WorldEvent) -> bool {
    let mut changed = false;
    if let WorldEventBody::Domain(domain) = event.body {
        match domain {
            super::super::events::DomainEvent::AgentRegistered { agent_id, pos } => {
                state
                    .agents
                    .entry(agent_id)
                    .and_modify(|entry| entry.pos = pos)
                    .or_insert(AgentPowerState { pos, level: 0 });
                changed = true;
            }
            super::super::events::DomainEvent::AgentMoved { agent_id, to, .. } => {
                if let Some(entry) = state.agents.get_mut(&agent_id) {
                    entry.pos = to;
                    changed = true;
                }
            }
            super::super::events::DomainEvent::ActionRejected { .. } => {}
            super::super::events::DomainEvent::Observation { .. } => {}
            super::super::events::DomainEvent::BodyAttributesUpdated { .. } => {}
            super::super::events::DomainEvent::BodyAttributesRejected { .. } => {}
            super::super::events::DomainEvent::BodyInterfaceExpanded { .. } => {}
            super::super::events::DomainEvent::BodyInterfaceExpandRejected { .. } => {}
            super::super::events::DomainEvent::ResourceTransferred { .. } => {}
        }
    }
    changed
}

fn radiation_harvest_per_tick(
    pos: GeoPos,
    base_per_tick: i64,
    distance_step_cm: i64,
    bonus_cap: i64,
) -> i64 {
    if base_per_tick <= 0 {
        return 0;
    }
    let axis_sum_cm = pos.x_cm.abs() + pos.y_cm.abs() + pos.z_cm.abs();
    let step = distance_step_cm.max(1) as f64;
    let bonus = (axis_sum_cm / step).floor() as i64;
    let bounded_bonus = bonus.clamp(0, bonus_cap.max(0));
    base_per_tick.saturating_add(bounded_bonus)
}

fn decode_state<T: DeserializeOwned + Default>(
    state: Option<&[u8]>,
    request: &ModuleCallRequest,
) -> Result<T, ModuleCallFailure> {
    let Some(state) = state else {
        return Ok(T::default());
    };
    if state.is_empty() {
        return Ok(T::default());
    }
    decode_input(request, state)
}

fn encode_state<T: Serialize>(
    state: &T,
    request: &ModuleCallRequest,
) -> Result<Vec<u8>, ModuleCallFailure> {
    to_canonical_cbor(state).map_err(|err| {
        failure(
            request,
            ModuleCallErrorCode::InvalidOutput,
            format!("state encode failed: {err:?}"),
        )
    })
}

fn decode_input<T: DeserializeOwned>(
    request: &ModuleCallRequest,
    bytes: &[u8],
) -> Result<T, ModuleCallFailure> {
    serde_cbor::from_slice(bytes).map_err(|err| {
        failure(
            request,
            ModuleCallErrorCode::InvalidOutput,
            format!("input CBOR decode failed: {err}"),
        )
    })
}

fn finalize_output(
    mut output: ModuleOutput,
    request: &ModuleCallRequest,
) -> Result<ModuleOutput, ModuleCallFailure> {
    output.output_bytes = 0;
    let encoded = serde_cbor::to_vec(&output).map_err(|err| {
        failure(
            request,
            ModuleCallErrorCode::InvalidOutput,
            format!("output encode failed: {err}"),
        )
    })?;
    output.output_bytes = encoded.len() as u64;
    Ok(output)
}

fn failure(
    request: &ModuleCallRequest,
    code: ModuleCallErrorCode,
    detail: String,
) -> ModuleCallFailure {
    ModuleCallFailure {
        module_id: request.module_id.clone(),
        trace_id: request.trace_id.clone(),
        code,
        detail,
    }
}

fn movement_cost(distance_cm: i64, per_km_cost: i64) -> i64 {
    if distance_cm <= 0 || per_km_cost <= 0 {
        return 0;
    }
    let km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
    km.saturating_mul(per_km_cost)
}
