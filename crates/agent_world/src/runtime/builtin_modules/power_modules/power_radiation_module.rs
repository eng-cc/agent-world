use serde_json::json;

use super::super::super::events::ActionEnvelope;
use super::super::super::sandbox::{
    ModuleCallFailure, ModuleCallInput, ModuleCallRequest, ModuleEmit, ModuleOutput,
};
use super::super::super::world_event::WorldEvent;
use super::super::BuiltinModule;
use super::{
    decode_input, decode_state, encode_state, finalize_output, radiation_harvest_per_tick,
    update_power_state_positions, PowerState, M1_POWER_HARVEST_BASE_PER_TICK,
    M1_POWER_HARVEST_DISTANCE_BONUS_CAP, M1_POWER_HARVEST_DISTANCE_STEP_CM,
};

#[derive(Debug, Clone)]
pub struct M1RadiationPowerModule {
    base_per_tick: i64,
    distance_step_cm: i64,
    distance_bonus_cap: i64,
}

impl Default for M1RadiationPowerModule {
    fn default() -> Self {
        Self {
            base_per_tick: M1_POWER_HARVEST_BASE_PER_TICK,
            distance_step_cm: M1_POWER_HARVEST_DISTANCE_STEP_CM,
            distance_bonus_cap: M1_POWER_HARVEST_DISTANCE_BONUS_CAP,
        }
    }
}

impl M1RadiationPowerModule {
    fn handle_action(
        &self,
        request: &ModuleCallRequest,
        envelope: ActionEnvelope,
        mut state: PowerState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let mut changed = false;
        for agent_state in state.agents.values_mut() {
            let harvested = radiation_harvest_per_tick(
                agent_state.pos,
                self.base_per_tick,
                self.distance_step_cm,
                self.distance_bonus_cap,
            );
            if harvested <= 0 {
                continue;
            }
            agent_state.level = agent_state.level.saturating_add(harvested);
            changed = true;
        }

        let new_state = if changed {
            Some(encode_state(&state, request)?)
        } else {
            None
        };

        let emit_payload = json!({
            "action_id": envelope.id,
            "agents": state
                .agents
                .iter()
                .map(|(agent_id, power)| {
                    json!({
                        "agent_id": agent_id,
                        "level": power.level,
                    })
                })
                .collect::<Vec<_>>()
        });

        finalize_output(
            ModuleOutput {
                new_state,
                effects: Vec::new(),
                emits: vec![ModuleEmit {
                    kind: "power.radiation_harvest".to_string(),
                    payload: emit_payload,
                }],
                output_bytes: 0,
            },
            request,
        )
    }

    fn handle_event(
        &self,
        request: &ModuleCallRequest,
        event: WorldEvent,
        mut state: PowerState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let changed = update_power_state_positions(&mut state, event);
        let new_state = if changed {
            Some(encode_state(&state, request)?)
        } else {
            None
        };

        finalize_output(
            ModuleOutput {
                new_state,
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}

impl BuiltinModule for M1RadiationPowerModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;
        let state: PowerState = decode_state(input.state.as_deref(), request)?;

        if let Some(action_bytes) = input.action.as_deref() {
            let envelope = decode_input::<ActionEnvelope>(request, action_bytes)?;
            return self.handle_action(request, envelope, state);
        }

        if let Some(event_bytes) = input.event.as_deref() {
            let event = decode_input::<WorldEvent>(request, event_bytes)?;
            return self.handle_event(request, event, state);
        }

        finalize_output(
            ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}
