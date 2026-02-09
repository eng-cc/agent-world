use bevy::prelude::*;

use super::{ViewerControl, ViewerState};

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StepControlLoadingState {
    pub pending: bool,
    pub baseline_tick: u64,
    pub baseline_event_count: usize,
    pub baseline_trace_count: usize,
}

pub(super) fn track_step_loading_state(
    mut loading: ResMut<StepControlLoadingState>,
    state: Res<ViewerState>,
) {
    if !loading.pending || !state.is_changed() {
        return;
    }

    let tick_advanced = state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.time > loading.baseline_tick)
        .unwrap_or(false)
        || state
            .metrics
            .as_ref()
            .map(|metrics| metrics.total_ticks > loading.baseline_tick)
            .unwrap_or(false);

    let events_advanced = state.events.len() > loading.baseline_event_count;
    let traces_advanced = state.decision_traces.len() > loading.baseline_trace_count;

    if tick_advanced || events_advanced || traces_advanced {
        loading.pending = false;
        return;
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
                .or_else(|| state.metrics.as_ref().map(|metrics| metrics.total_ticks))
                .unwrap_or(0);
            loading.baseline_event_count = state.events.len();
            loading.baseline_trace_count = state.decision_traces.len();
        }
        ViewerControl::Play | ViewerControl::Pause | ViewerControl::Seek { .. } => {
            loading.pending = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::simulator::{AgentDecision, AgentDecisionTrace};
    use bevy::app::Update;

    fn sample_snapshot(time: u64) -> agent_world::simulator::WorldSnapshot {
        agent_world::simulator::WorldSnapshot {
            version: agent_world::simulator::SNAPSHOT_VERSION,
            chunk_generation_schema_version:
                agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
            time,
            config: agent_world::simulator::WorldConfig::default(),
            model: agent_world::simulator::WorldModel::default(),
            chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
            next_event_id: 1,
            next_action_id: 1,
            pending_actions: Vec::new(),
            journal_len: 0,
        }
    }

    fn sample_trace(time: u64, llm_error: Option<&str>) -> AgentDecisionTrace {
        AgentDecisionTrace {
            agent_id: "agent-1".to_string(),
            time,
            decision: AgentDecision::Wait,
            llm_input: None,
            llm_output: None,
            llm_error: llm_error.map(str::to_string),
            parse_error: None,
            llm_diagnostics: None,
            llm_effect_intents: Vec::new(),
            llm_effect_receipts: Vec::new(),
            llm_step_trace: Vec::new(),
            llm_prompt_section_trace: Vec::new(),
        }
    }

    #[test]
    fn mark_step_loading_uses_snapshot_tick_baseline() {
        let state = ViewerState {
            status: super::super::ConnectionStatus::Connected,
            snapshot: Some(sample_snapshot(11)),
            events: Vec::new(),
            decision_traces: Vec::new(),
            metrics: None,
        };
        let mut loading = StepControlLoadingState::default();

        mark_step_loading_on_control(&ViewerControl::Step { count: 1 }, &state, &mut loading);

        assert!(loading.pending);
        assert_eq!(loading.baseline_tick, 11);
        assert_eq!(loading.baseline_event_count, 0);
        assert_eq!(loading.baseline_trace_count, 0);

        mark_step_loading_on_control(&ViewerControl::Pause, &state, &mut loading);
        assert!(!loading.pending);
    }

    #[test]
    fn track_step_loading_clears_pending_when_llm_trace_arrives_without_tick_advance() {
        let mut app = App::new();
        app.add_systems(Update, track_step_loading_state);
        app.world_mut().insert_resource(StepControlLoadingState {
            pending: true,
            baseline_tick: 11,
            baseline_event_count: 0,
            baseline_trace_count: 0,
        });
        app.world_mut().insert_resource(ViewerState {
            status: super::super::ConnectionStatus::Connected,
            snapshot: Some(sample_snapshot(11)),
            events: Vec::new(),
            decision_traces: vec![sample_trace(11, Some("llm timeout"))],
            metrics: None,
        });

        app.update();

        let loading = app.world().resource::<StepControlLoadingState>();
        assert!(!loading.pending);
    }
}
