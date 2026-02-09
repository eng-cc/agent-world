use bevy::prelude::*;

use super::{ViewerControl, ViewerState};

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StepControlLoadingState {
    pub pending: bool,
    pub baseline_tick: u64,
}

pub(super) fn track_step_loading_state(
    mut loading: ResMut<StepControlLoadingState>,
    state: Res<ViewerState>,
) {
    if !loading.pending || !state.is_changed() {
        return;
    }

    if let Some(snapshot) = state.snapshot.as_ref() {
        if snapshot.time > loading.baseline_tick {
            loading.pending = false;
        }
    }

    if matches!(state.status, super::ConnectionStatus::Error(_)) {
        loading.pending = false;
    }
}

pub(super) fn mark_step_loading_on_control(
    control: &ViewerControl,
    state: &ViewerState,
    loading: &mut StepControlLoadingState,
) {
    match control {
        ViewerControl::Step { .. } => {
            if loading.pending {
                return;
            }
            loading.pending = true;
            loading.baseline_tick = state
                .snapshot
                .as_ref()
                .map(|snapshot| snapshot.time)
                .unwrap_or(0);
        }
        ViewerControl::Play | ViewerControl::Pause | ViewerControl::Seek { .. } => {
            loading.pending = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_step_loading_uses_snapshot_tick_baseline() {
        let state = ViewerState {
            status: super::super::ConnectionStatus::Connected,
            snapshot: Some(agent_world::simulator::WorldSnapshot {
                version: agent_world::simulator::SNAPSHOT_VERSION,
                chunk_generation_schema_version:
                    agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
                time: 11,
                config: agent_world::simulator::WorldConfig::default(),
                model: agent_world::simulator::WorldModel::default(),
                chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
                next_event_id: 1,
                next_action_id: 1,
                pending_actions: Vec::new(),
                journal_len: 0,
            }),
            events: Vec::new(),
            decision_traces: Vec::new(),
            metrics: None,
        };
        let mut loading = StepControlLoadingState::default();

        mark_step_loading_on_control(&ViewerControl::Step { count: 1 }, &state, &mut loading);

        assert!(loading.pending);
        assert_eq!(loading.baseline_tick, 11);

        mark_step_loading_on_control(&ViewerControl::Pause, &state, &mut loading);
        assert!(!loading.pending);
    }
}
