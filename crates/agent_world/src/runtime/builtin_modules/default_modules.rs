use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::super::events::DomainEvent;
use super::super::sandbox::{ModuleCallFailure, ModuleCallInput, ModuleCallRequest, ModuleOutput};
use super::super::world_event::{WorldEvent, WorldEventBody};
use super::{
    decode_input, decode_state, encode_state, finalize_output, BuiltinModule, M1MoveRuleModule,
    M1VisibilityRuleModule, M1_MEMORY_MAX_ENTRIES,
};

#[derive(Debug, Clone, Default)]
pub struct M1SensorModule {
    inner: M1VisibilityRuleModule,
}

impl M1SensorModule {
    pub fn new(visibility_range_cm: i64) -> Self {
        Self {
            inner: M1VisibilityRuleModule::new(visibility_range_cm),
        }
    }
}

impl BuiltinModule for M1SensorModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        self.inner.call(request)
    }
}

#[derive(Debug, Clone, Default)]
pub struct M1MobilityModule {
    inner: M1MoveRuleModule,
}

impl M1MobilityModule {
    pub fn new(per_km_cost: i64) -> Self {
        Self {
            inner: M1MoveRuleModule::new(per_km_cost),
        }
    }
}

impl BuiltinModule for M1MobilityModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        self.inner.call(request)
    }
}

#[derive(Debug, Clone)]
pub struct M1MemoryModule {
    max_entries: usize,
}

impl Default for M1MemoryModule {
    fn default() -> Self {
        Self {
            max_entries: M1_MEMORY_MAX_ENTRIES,
        }
    }
}

impl M1MemoryModule {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries: max_entries.max(1),
        }
    }

    fn handle_event(
        &self,
        request: &ModuleCallRequest,
        event: WorldEvent,
        mut state: MemoryState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let WorldEventBody::Domain(domain) = event.body else {
            return finalize_output(
                ModuleOutput {
                    new_state: None,
                    effects: Vec::new(),
                    emits: Vec::new(),
                    output_bytes: 0,
                },
                request,
            );
        };

        let (kind, agent_id) = memory_domain_label(&domain);
        state.entries.push(MemoryEntry {
            time: event.time,
            kind: kind.to_string(),
            agent_id,
        });
        if state.entries.len() > self.max_entries {
            let overflow = state.entries.len() - self.max_entries;
            state.entries.drain(0..overflow);
        }

        finalize_output(
            ModuleOutput {
                new_state: Some(encode_state(&state, request)?),
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}

impl BuiltinModule for M1MemoryModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;
        let state: MemoryState = decode_state(input.state.as_deref(), request)?;

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

#[derive(Debug, Clone, Default)]
pub struct M1StorageCargoModule;

impl BuiltinModule for M1StorageCargoModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;
        let mut state: CargoLedgerState = decode_state(input.state.as_deref(), request)?;

        let Some(event_bytes) = input.event.as_deref() else {
            return finalize_output(
                ModuleOutput {
                    new_state: None,
                    effects: Vec::new(),
                    emits: Vec::new(),
                    output_bytes: 0,
                },
                request,
            );
        };
        let event = decode_input::<WorldEvent>(request, event_bytes)?;
        let WorldEventBody::Domain(domain) = event.body else {
            return finalize_output(
                ModuleOutput {
                    new_state: None,
                    effects: Vec::new(),
                    emits: Vec::new(),
                    output_bytes: 0,
                },
                request,
            );
        };

        let mut changed = false;
        match domain {
            DomainEvent::BodyInterfaceExpanded {
                agent_id,
                consumed_item_id,
                expansion_level,
                ..
            } => {
                state
                    .agent_expansion_levels
                    .insert(agent_id, expansion_level);
                let entry = state
                    .consumed_interface_items
                    .entry(consumed_item_id)
                    .or_insert(0);
                *entry = entry.saturating_add(1);
                changed = true;
            }
            DomainEvent::BodyInterfaceExpandRejected { .. } => {
                state.reject_count = state.reject_count.saturating_add(1);
                changed = true;
            }
            _ => {}
        }

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MemoryState {
    entries: Vec<MemoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryEntry {
    time: u64,
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CargoLedgerState {
    consumed_interface_items: BTreeMap<String, u64>,
    agent_expansion_levels: BTreeMap<String, u16>,
    reject_count: u64,
}

fn memory_domain_label(domain: &DomainEvent) -> (&'static str, Option<String>) {
    match domain {
        DomainEvent::AgentRegistered { agent_id, .. } => {
            ("domain.agent_registered", Some(agent_id.clone()))
        }
        DomainEvent::AgentMoved { agent_id, .. } => ("domain.agent_moved", Some(agent_id.clone())),
        DomainEvent::ActionRejected { .. } => ("domain.action_rejected", None),
        DomainEvent::Observation { observation } => {
            ("domain.observation", Some(observation.agent_id.clone()))
        }
        DomainEvent::BodyAttributesUpdated { agent_id, .. } => {
            ("domain.body_attributes_updated", Some(agent_id.clone()))
        }
        DomainEvent::BodyAttributesRejected { agent_id, .. } => {
            ("domain.body_attributes_rejected", Some(agent_id.clone()))
        }
        DomainEvent::BodyInterfaceExpanded { agent_id, .. } => {
            ("domain.body_interface_expanded", Some(agent_id.clone()))
        }
        DomainEvent::BodyInterfaceExpandRejected { agent_id, .. } => (
            "domain.body_interface_expand_rejected",
            Some(agent_id.clone()),
        ),
        DomainEvent::ResourceTransferred { from_agent_id, .. } => {
            ("domain.resource_transferred", Some(from_agent_id.clone()))
        }
    }
}
