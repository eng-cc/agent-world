use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use crate::geometry::GeoPos;
use crate::runtime::{
    blake3_hex, Action as RuntimeAction, DomainEvent as RuntimeDomainEvent,
    Journal as RuntimeJournal, Snapshot as RuntimeSnapshot, World as RuntimeWorld,
    WorldError as RuntimeWorldError, WorldEventBody as RuntimeWorldEventBody,
};
use crate::simulator::{
    build_world_model, AgentDecisionTrace, ChunkRuntimeConfig, PlayerGameplayRecentFeedback,
    RejectReason as SimulatorRejectReason, ResourceKind, RunnerMetrics, WorldConfig, WorldEvent,
    WorldInitConfig, WorldScenario, WorldSnapshot, CHUNK_GENERATION_SCHEMA_VERSION,
    SNAPSHOT_VERSION,
};

use super::auth::verify_session_register_auth_proof;
use super::live::ViewerLiveDecisionMode;
use super::protocol::{
    viewer_event_kind_matches, AuthoritativeBatchFinality, AuthoritativeChallengeAck,
    AuthoritativeChallengeCommand, AuthoritativeChallengeError,
    AuthoritativeChallengeResolveRequest, AuthoritativeChallengeStatus,
    AuthoritativeChallengeSubmitRequest, AuthoritativeFinalityState,
    AuthoritativeReconnectSyncRequest, AuthoritativeRecoveryAck, AuthoritativeRecoveryCommand,
    AuthoritativeRecoveryError, AuthoritativeRecoveryStatus, AuthoritativeRollbackRequest,
    AuthoritativeSessionRegisterRequest, AuthoritativeSessionRevokeRequest,
    AuthoritativeSessionRotateRequest, ControlCompletionAck, ControlCompletionStatus,
    ViewerControl, ViewerControlProfile, ViewerEventKind, ViewerRequest, ViewerResponse,
    ViewerStream, VIEWER_PROTOCOL_VERSION,
};
#[path = "runtime_live/control_plane.rs"]
mod control_plane;
mod gameplay_snapshot;
mod mapping;
mod player_gameplay;
#[cfg(test)]
mod tests;

use control_plane::RuntimeLlmSidecar;
use gameplay_snapshot::{
    build_player_gameplay_snapshot, player_gameplay_feedback_from_control_ack,
};
use mapping::{map_runtime_event, runtime_state_to_simulator_model};

const AUTHORITATIVE_BATCH_CONFIRM_DELAY_TICKS: u64 = 1;
const AUTHORITATIVE_BATCH_FINALITY_WINDOW_TICKS: u64 = 2;
const MAX_AUTHORITATIVE_BATCH_HISTORY: usize = 256;
const MAX_AUTHORITATIVE_CHALLENGE_HISTORY: usize = 512;
const MAX_AUTHORITATIVE_STABLE_CHECKPOINTS: usize = 64;

#[derive(Debug, Clone)]
struct RuntimeStableCheckpoint {
    batch_id: String,
    snapshot: RuntimeSnapshot,
    journal: RuntimeJournal,
    log_cursor: u64,
}

#[derive(Debug, Clone)]
struct RuntimeRecoveryCursor {
    snapshot_hash: String,
    snapshot_height: u64,
    log_cursor: u64,
    stable_batch_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct RuntimeSessionPolicy {
    active_pubkey_by_player: BTreeMap<String, String>,
    revoked_pubkeys_by_player: BTreeMap<String, BTreeSet<String>>,
    session_epoch_by_player: BTreeMap<String, u64>,
}

impl RuntimeSessionPolicy {
    fn register_session(&mut self, player_id: &str, public_key: &str) -> Result<u64, String> {
        let player_id = player_id.trim();
        let public_key = public_key.trim();
        if player_id.is_empty() {
            return Err("session_player_id_invalid: player_id cannot be empty".to_string());
        }
        if public_key.is_empty() {
            return Err("session_pubkey_invalid: session_pubkey cannot be empty".to_string());
        }
        if self
            .revoked_pubkeys_by_player
            .get(player_id)
            .is_some_and(|keys| keys.contains(public_key))
        {
            return Err(format!(
                "session_revoked: player {} session_pubkey {} is revoked",
                player_id, public_key
            ));
        }

        match self.active_pubkey_by_player.get(player_id) {
            Some(active) if active == public_key => {}
            Some(active) => {
                return Err(format!(
                    "session_key_mismatch: player {} active session_pubkey {} does not match {}",
                    player_id, active, public_key
                ));
            }
            None => {
                self.active_pubkey_by_player
                    .insert(player_id.to_string(), public_key.to_string());
                self.session_epoch_by_player
                    .entry(player_id.to_string())
                    .or_insert(1);
            }
        }

        Ok(self.session_epoch(player_id))
    }

    fn validate_known_session_key(&self, player_id: &str, public_key: &str) -> Result<u64, String> {
        let player_id = player_id.trim();
        let public_key = public_key.trim();
        if player_id.is_empty() {
            return Err("session_player_id_invalid: player_id cannot be empty".to_string());
        }
        if public_key.is_empty() {
            return Err("session_pubkey_invalid: session_pubkey cannot be empty".to_string());
        }

        if self
            .revoked_pubkeys_by_player
            .get(player_id)
            .is_some_and(|keys| keys.contains(public_key))
        {
            return Err(format!(
                "session_revoked: player {} session_pubkey {} is revoked",
                player_id, public_key
            ));
        }
        match self.active_pubkey_by_player.get(player_id) {
            Some(active) if active == public_key => {}
            Some(active) => {
                return Err(format!(
                    "session_key_mismatch: player {} active session_pubkey {} does not match {}",
                    player_id, active, public_key
                ));
            }
            None => {
                return Err(format!(
                    "session_not_found: player {} has no active session_pubkey",
                    player_id
                ));
            }
        }
        Ok(self.session_epoch(player_id))
    }

    fn revoke_session(
        &mut self,
        player_id: &str,
        session_pubkey: Option<&str>,
    ) -> Result<(String, u64), String> {
        let player_id = player_id.trim();
        if player_id.is_empty() {
            return Err("session_player_id_invalid: player_id cannot be empty".to_string());
        }

        let target = session_pubkey
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .or_else(|| self.active_pubkey_by_player.get(player_id).cloned())
            .ok_or_else(|| {
                format!(
                    "session_not_found: player {} has no active session_pubkey",
                    player_id
                )
            })?;

        let revoked = self
            .revoked_pubkeys_by_player
            .entry(player_id.to_string())
            .or_default()
            .insert(target.clone());
        if self
            .active_pubkey_by_player
            .get(player_id)
            .is_some_and(|active| active == &target)
        {
            self.active_pubkey_by_player.remove(player_id);
        }

        if revoked {
            let next_epoch = self.session_epoch(player_id).saturating_add(1).max(1);
            self.session_epoch_by_player
                .insert(player_id.to_string(), next_epoch);
        }

        Ok((target, self.session_epoch(player_id)))
    }

    fn rotate_session(
        &mut self,
        player_id: &str,
        old_session_pubkey: &str,
        new_session_pubkey: &str,
    ) -> Result<u64, String> {
        let player_id = player_id.trim();
        let old_session_pubkey = old_session_pubkey.trim();
        let new_session_pubkey = new_session_pubkey.trim();
        if player_id.is_empty() {
            return Err("session_player_id_invalid: player_id cannot be empty".to_string());
        }
        if old_session_pubkey.is_empty() || new_session_pubkey.is_empty() {
            return Err("session_pubkey_invalid: session_pubkey cannot be empty".to_string());
        }
        if old_session_pubkey == new_session_pubkey {
            return Err("session_rotation_invalid: old/new session_pubkey must differ".to_string());
        }

        if self
            .active_pubkey_by_player
            .get(player_id)
            .is_some_and(|active| active != old_session_pubkey)
        {
            return Err(format!(
                "session_key_mismatch: player {} active session_pubkey does not match {}",
                player_id, old_session_pubkey
            ));
        }
        if self
            .revoked_pubkeys_by_player
            .get(player_id)
            .is_some_and(|keys| keys.contains(new_session_pubkey))
        {
            return Err(format!(
                "session_rotation_invalid: new session_pubkey {} is already revoked",
                new_session_pubkey
            ));
        }

        self.revoked_pubkeys_by_player
            .entry(player_id.to_string())
            .or_default()
            .insert(old_session_pubkey.to_string());
        self.active_pubkey_by_player
            .insert(player_id.to_string(), new_session_pubkey.to_string());
        let next_epoch = self.session_epoch(player_id).saturating_add(1).max(1);
        self.session_epoch_by_player
            .insert(player_id.to_string(), next_epoch);
        Ok(next_epoch)
    }

    fn session_epoch(&self, player_id: &str) -> u64 {
        self.session_epoch_by_player
            .get(player_id)
            .copied()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeBatchChallengeState {
    None,
    Challenged,
    ResolvedNoFraud,
    ResolvedFraudSlashed,
}

#[derive(Debug, Clone)]
struct RuntimeAuthoritativeChallengeRecord {
    challenge_id: String,
    batch_id: String,
    watcher_id: String,
    recomputed_state_root: String,
    recomputed_data_root: String,
    status: AuthoritativeChallengeStatus,
    submitted_at_tick: u64,
    resolved_at_tick: Option<u64>,
    slash_applied: bool,
    slash_reason: Option<String>,
}

impl RuntimeAuthoritativeChallengeRecord {
    fn as_ack(&self) -> AuthoritativeChallengeAck<u64> {
        AuthoritativeChallengeAck {
            challenge_id: self.challenge_id.clone(),
            batch_id: self.batch_id.clone(),
            watcher_id: self.watcher_id.clone(),
            status: self.status,
            submitted_at_tick: self.submitted_at_tick,
            resolved_at_tick: self.resolved_at_tick,
            slash_applied: self.slash_applied,
            slash_reason: self.slash_reason.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeAuthoritativeBatchRecord {
    batch_id: String,
    tx_hash: String,
    commit_tick: u64,
    confirm_height: u64,
    final_height: u64,
    state_root: String,
    data_root: String,
    event_seq_start: Option<u64>,
    event_seq_end: Option<u64>,
    finality_state: AuthoritativeFinalityState,
    challenge_state: RuntimeBatchChallengeState,
    active_challenge_id: Option<String>,
    events: Vec<WorldEvent>,
}

impl RuntimeAuthoritativeBatchRecord {
    fn has_valid_commit_roots(&self) -> bool {
        is_valid_root_hash(self.state_root.as_str()) && is_valid_root_hash(self.data_root.as_str())
    }

    fn expected_finality(&self, current_tick: u64) -> AuthoritativeFinalityState {
        if current_tick >= self.final_height {
            AuthoritativeFinalityState::Final
        } else if current_tick >= self.confirm_height {
            AuthoritativeFinalityState::Confirmed
        } else {
            AuthoritativeFinalityState::Pending
        }
    }

    fn as_wire(&self, gate: &RuntimeSettlementRankingGate) -> AuthoritativeBatchFinality {
        AuthoritativeBatchFinality {
            batch_id: self.batch_id.clone(),
            tx_hash: self.tx_hash.clone(),
            commit_tick: self.commit_tick,
            confirm_height: self.confirm_height,
            final_height: self.final_height,
            state_root: self.state_root.clone(),
            data_root: self.data_root.clone(),
            finality_state: self.finality_state,
            event_seq_start: self.event_seq_start,
            event_seq_end: self.event_seq_end,
            settlement_ready: gate.settlement_allowed(self.batch_id.as_str(), self.finality_state),
            ranking_ready: gate.ranking_allowed(self.batch_id.as_str(), self.finality_state),
            challenge_open: self.challenge_state == RuntimeBatchChallengeState::Challenged,
            slashed: self.challenge_state == RuntimeBatchChallengeState::ResolvedFraudSlashed,
            active_challenge_id: self.active_challenge_id.clone(),
        }
    }
}

#[derive(Debug, Default)]
struct RuntimeSettlementRankingGate {
    settlement_ready_batches: BTreeSet<String>,
    ranking_ready_batches: BTreeSet<String>,
}

impl RuntimeSettlementRankingGate {
    fn promote_final(&mut self, batch_id: &str) {
        self.settlement_ready_batches.insert(batch_id.to_string());
        self.ranking_ready_batches.insert(batch_id.to_string());
    }

    fn evict_batch(&mut self, batch_id: &str) {
        self.settlement_ready_batches.remove(batch_id);
        self.ranking_ready_batches.remove(batch_id);
    }

    fn settlement_allowed(
        &self,
        batch_id: &str,
        finality_state: AuthoritativeFinalityState,
    ) -> bool {
        finality_state == AuthoritativeFinalityState::Final
            && self.settlement_ready_batches.contains(batch_id)
    }

    fn ranking_allowed(&self, batch_id: &str, finality_state: AuthoritativeFinalityState) -> bool {
        finality_state == AuthoritativeFinalityState::Final
            && self.ranking_ready_batches.contains(batch_id)
    }
}

#[derive(Debug, Clone)]
pub struct ViewerRuntimeLiveServerConfig {
    pub bind_addr: String,
    pub scenario: WorldScenario,
    pub world_id: String,
    pub decision_mode: ViewerLiveDecisionMode,
    pub play_step_interval: Duration,
    pub hosted_public_join_mode: bool,
}

impl ViewerRuntimeLiveServerConfig {
    pub fn new(scenario: WorldScenario) -> Self {
        Self {
            bind_addr: "127.0.0.1:5010".to_string(),
            world_id: format!("live-runtime-{}", scenario.as_str()),
            scenario,
            decision_mode: ViewerLiveDecisionMode::Script,
            play_step_interval: Duration::from_millis(800),
            hosted_public_join_mode: false,
        }
    }

    pub fn with_bind_addr(mut self, addr: impl Into<String>) -> Self {
        self.bind_addr = addr.into();
        self
    }

    pub fn with_world_id(mut self, world_id: impl Into<String>) -> Self {
        self.world_id = world_id.into();
        self
    }

    pub fn with_decision_mode(mut self, mode: ViewerLiveDecisionMode) -> Self {
        self.decision_mode = mode;
        self
    }

    pub fn with_llm_mode(mut self, enabled: bool) -> Self {
        self.decision_mode = if enabled {
            ViewerLiveDecisionMode::Llm
        } else {
            ViewerLiveDecisionMode::Script
        };
        self
    }

    pub fn with_play_step_interval(mut self, interval: Duration) -> Self {
        self.play_step_interval = interval.max(Duration::from_millis(50));
        self
    }

    pub fn with_hosted_public_join_mode(mut self, enabled: bool) -> Self {
        self.hosted_public_join_mode = enabled;
        self
    }
}

#[derive(Debug)]
pub enum ViewerRuntimeLiveServerError {
    Io(io::Error),
    Serde(String),
    Init(String),
    Runtime(RuntimeWorldError),
}

impl From<io::Error> for ViewerRuntimeLiveServerError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<RuntimeWorldError> for ViewerRuntimeLiveServerError {
    fn from(err: RuntimeWorldError) -> Self {
        Self::Runtime(err)
    }
}

pub struct ViewerRuntimeLiveServer {
    config: ViewerRuntimeLiveServerConfig,
    world: RuntimeWorld,
    snapshot_config: WorldConfig,
    script: RuntimeLiveScript,
    llm_sidecar: RuntimeLlmSidecar,
    pending_virtual_events: VecDeque<WorldEvent>,
    next_virtual_event_id: u64,
    authoritative_batches: VecDeque<RuntimeAuthoritativeBatchRecord>,
    next_authoritative_batch_id: u64,
    authoritative_challenges: VecDeque<RuntimeAuthoritativeChallengeRecord>,
    next_authoritative_challenge_id: u64,
    stable_checkpoints: VecDeque<RuntimeStableCheckpoint>,
    reorg_epoch: u64,
    session_policy: RuntimeSessionPolicy,
    settlement_ranking_gate: RuntimeSettlementRankingGate,
    latest_player_gameplay_feedback: Option<PlayerGameplayRecentFeedback>,
}

impl ViewerRuntimeLiveServer {
    pub fn new(
        config: ViewerRuntimeLiveServerConfig,
    ) -> Result<Self, ViewerRuntimeLiveServerError> {
        let (world, snapshot_config) =
            bootstrap_runtime_world(config.scenario).map_err(ViewerRuntimeLiveServerError::Init)?;
        let llm_sidecar = RuntimeLlmSidecar::new(config.decision_mode);
        let next_virtual_event_id = latest_runtime_event_seq(&world).saturating_add(1).max(1);
        Ok(Self {
            config,
            world,
            snapshot_config,
            script: RuntimeLiveScript::default(),
            llm_sidecar,
            pending_virtual_events: VecDeque::new(),
            next_virtual_event_id,
            authoritative_batches: VecDeque::new(),
            next_authoritative_batch_id: 1,
            authoritative_challenges: VecDeque::new(),
            next_authoritative_challenge_id: 1,
            stable_checkpoints: VecDeque::new(),
            reorg_epoch: 0,
            session_policy: RuntimeSessionPolicy::default(),
            settlement_ranking_gate: RuntimeSettlementRankingGate::default(),
            latest_player_gameplay_feedback: None,
        })
    }

    pub fn run(&mut self) -> Result<(), ViewerRuntimeLiveServerError> {
        let listener = TcpListener::bind(&self.config.bind_addr)?;
        for incoming in listener.incoming() {
            let stream = incoming?;
            if let Err(err) = self.serve_stream(stream) {
                eprintln!("viewer runtime live server error: {err:?}");
            }
        }
        Ok(())
    }

    pub fn run_once(&mut self) -> Result<(), ViewerRuntimeLiveServerError> {
        let listener = TcpListener::bind(&self.config.bind_addr)?;
        let (stream, _) = listener.accept()?;
        self.serve_stream(stream)
    }

    fn hosted_public_join_mode(&self) -> bool {
        self.config.hosted_public_join_mode
    }

    fn serve_stream(&mut self, stream: TcpStream) -> Result<(), ViewerRuntimeLiveServerError> {
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_millis(50)))?;

        let reader_stream = stream.try_clone()?;
        let mut reader = BufReader::new(reader_stream);
        let mut writer = BufWriter::new(stream);
        let mut session = RuntimeLiveSession::new();

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return Ok(()),
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        match serde_json::from_str::<ViewerRequest>(trimmed) {
                            Ok(request) => {
                                self.handle_request(request, &mut session, &mut writer)?;
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(err) if is_timeout_error(&err) => {}
                Err(err) => return Err(ViewerRuntimeLiveServerError::Io(err)),
            }

            if session.should_advance_play_step(self.config.play_step_interval) {
                self.advance_runtime(&mut session, &mut writer, 1, None, false)?;
            }
        }
    }

    fn handle_request(
        &mut self,
        request: ViewerRequest,
        session: &mut RuntimeLiveSession,
        writer: &mut BufWriter<TcpStream>,
    ) -> Result<(), ViewerRuntimeLiveServerError> {
        match request {
            ViewerRequest::Hello { .. } => {
                send_response(
                    writer,
                    &ViewerResponse::HelloAck {
                        server: "oasis7".to_string(),
                        version: VIEWER_PROTOCOL_VERSION,
                        world_id: self.config.world_id.clone(),
                        control_profile: ViewerControlProfile::Live,
                    },
                )?;
            }
            ViewerRequest::Subscribe {
                streams,
                event_kinds,
            } => {
                session.subscribed = streams.into_iter().collect();
                session.event_filters = if event_kinds.is_empty() {
                    None
                } else {
                    Some(event_kinds.into_iter().collect())
                };
            }
            ViewerRequest::RequestSnapshot => {
                if session.subscribed.is_empty()
                    || session.subscribed.contains(&ViewerStream::Snapshot)
                {
                    let snapshot = self.compat_snapshot();
                    send_response(writer, &ViewerResponse::Snapshot { snapshot })?;
                }
                if session.subscribed.is_empty()
                    || session.subscribed.contains(&ViewerStream::Snapshot)
                    || session.subscribed.contains(&ViewerStream::Events)
                {
                    let cursor = self.current_recovery_cursor()?;
                    send_response(
                        writer,
                        &ViewerResponse::AuthoritativeRecoveryAck {
                            ack: AuthoritativeRecoveryAck {
                                status: AuthoritativeRecoveryStatus::CatchUpReady,
                                reorg_epoch: self.reorg_epoch,
                                snapshot_height: cursor.snapshot_height,
                                snapshot_hash: cursor.snapshot_hash,
                                log_cursor: cursor.log_cursor,
                                stable_batch_id: cursor.stable_batch_id,
                                player_id: None,
                                agent_id: None,
                                session_pubkey: None,
                                replaced_by_pubkey: None,
                                session_epoch: None,
                                message: Some("snapshot_sync_metadata".to_string()),
                                acknowledged_at_tick: self.world.state().time,
                            },
                        },
                    )?;
                }
                if session.subscribed.contains(&ViewerStream::Metrics) {
                    session.metrics = runtime_metrics(&self.world);
                    send_response(
                        writer,
                        &ViewerResponse::Metrics {
                            time: Some(self.world.state().time),
                            metrics: session.metrics.clone(),
                        },
                    )?;
                }
                if session.subscribed.contains(&ViewerStream::Events) {
                    self.emit_authoritative_batch_snapshot(writer)?;
                    self.emit_authoritative_challenge_snapshot(writer)?;
                }
            }
            ViewerRequest::PlaybackControl { mode, request_id } => {
                self.apply_control_mode(ViewerControl::from(mode), request_id, session, writer)?;
            }
            ViewerRequest::LiveControl { mode, request_id } => {
                self.apply_control_mode(ViewerControl::from(mode), request_id, session, writer)?;
            }
            ViewerRequest::Control { mode, request_id } => {
                self.apply_control_mode(mode, request_id, session, writer)?;
            }
            ViewerRequest::PromptControl { command } => match self.handle_prompt_control(command) {
                Ok(ack) => {
                    send_response(writer, &ViewerResponse::PromptControlAck { ack })?;
                }
                Err(error) => {
                    send_response(writer, &ViewerResponse::PromptControlError { error })?;
                }
            },
            ViewerRequest::AgentChat { request } => match self.handle_agent_chat(request) {
                Ok(ack) => {
                    send_response(writer, &ViewerResponse::AgentChatAck { ack })?;
                }
                Err(error) => {
                    send_response(writer, &ViewerResponse::AgentChatError { error })?;
                }
            },
            ViewerRequest::GameplayAction { request } => match self.handle_gameplay_action(request)
            {
                Ok(ack) => {
                    send_response(writer, &ViewerResponse::GameplayActionAck { ack })?;
                }
                Err(error) => {
                    send_response(writer, &ViewerResponse::GameplayActionError { error })?;
                }
            },
            ViewerRequest::AuthoritativeChallenge { command } => {
                match self.handle_authoritative_challenge(command) {
                    Ok((ack, maybe_batch_update)) => {
                        send_response(writer, &ViewerResponse::AuthoritativeChallengeAck { ack })?;
                        if let Some(batch) = maybe_batch_update {
                            send_response(writer, &ViewerResponse::AuthoritativeBatch { batch })?;
                        }
                    }
                    Err(error) => {
                        send_response(
                            writer,
                            &ViewerResponse::AuthoritativeChallengeError { error },
                        )?;
                    }
                }
            }
            ViewerRequest::AuthoritativeRecovery { command } => {
                match self.handle_authoritative_recovery(command) {
                    Ok((ack, emit_snapshot_after_ack)) => {
                        send_response(writer, &ViewerResponse::AuthoritativeRecoveryAck { ack })?;
                        if emit_snapshot_after_ack {
                            let snapshot = self.compat_snapshot();
                            send_response(writer, &ViewerResponse::Snapshot { snapshot })?;
                            self.emit_authoritative_batch_snapshot(writer)?;
                            self.emit_authoritative_challenge_snapshot(writer)?;
                        }
                    }
                    Err(error) => {
                        send_response(
                            writer,
                            &ViewerResponse::AuthoritativeRecoveryError { error },
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn apply_control_mode(
        &mut self,
        mode: ViewerControl,
        request_id: Option<u64>,
        session: &mut RuntimeLiveSession,
        writer: &mut BufWriter<TcpStream>,
    ) -> Result<(), ViewerRuntimeLiveServerError> {
        match mode {
            ViewerControl::Pause => {
                session.playing = false;
                session.next_play_step_at = None;
            }
            ViewerControl::Play => {
                session.playing = true;
                session.next_play_step_at = None;
            }
            ViewerControl::Step { count } => {
                session.playing = false;
                session.next_play_step_at = None;
                self.advance_runtime(session, writer, count.max(1), request_id, true)?;
            }
            ViewerControl::Seek { tick } => {
                session.playing = false;
                session.next_play_step_at = None;
                eprintln!(
                    "viewer runtime live: ignore seek control in live mode (target_tick={tick})"
                );
            }
        }
        Ok(())
    }

    fn advance_runtime(
        &mut self,
        session: &mut RuntimeLiveSession,
        writer: &mut BufWriter<TcpStream>,
        step_count: usize,
        request_id: Option<u64>,
        emit_while_paused: bool,
    ) -> Result<(), ViewerRuntimeLiveServerError> {
        let baseline_logical_time = self.world.state().time;
        let baseline_event_seq = latest_runtime_event_seq(&self.world);

        for _ in 0..step_count.max(1) {
            let mut decision_trace: Option<AgentDecisionTrace> = None;
            match self.config.decision_mode {
                ViewerLiveDecisionMode::Script => self.script.enqueue(&mut self.world),
                ViewerLiveDecisionMode::Llm => {
                    decision_trace = self.enqueue_llm_action_from_sidecar();
                }
            }
            let journal_start = self.world.journal().events.len();
            self.world.step()?;

            let new_events: Vec<_> = self.world.journal().events[journal_start..].to_vec();
            let mut mapped_events = Vec::new();
            for runtime_event in &new_events {
                let event = map_runtime_event(runtime_event, &self.snapshot_config);
                if matches!(runtime_event.body, RuntimeWorldEventBody::Domain(_)) {
                    self.llm_sidecar
                        .notify_action_result_if_needed(runtime_event, event.clone());
                }
                mapped_events.push(event);
            }
            mapped_events.extend(self.pending_virtual_events.drain(..));
            let pending_batch = self.register_authoritative_batch(mapped_events.as_slice())?;
            let batch_finality_updates =
                self.advance_authoritative_batch_finality(self.world.state().time)?;

            if let Some(trace) = decision_trace {
                if session.subscribed.contains(&ViewerStream::Events) {
                    send_response(writer, &ViewerResponse::DecisionTrace { trace })?;
                }
            }

            if session.subscribed.contains(&ViewerStream::Events)
                && (emit_while_paused || session.playing)
            {
                for event in &mapped_events {
                    if session.event_allowed(event) {
                        send_response(
                            writer,
                            &ViewerResponse::Event {
                                event: event.clone(),
                            },
                        )?;
                    }
                }
                send_response(
                    writer,
                    &ViewerResponse::AuthoritativeBatch {
                        batch: pending_batch,
                    },
                )?;
                for batch in batch_finality_updates {
                    send_response(writer, &ViewerResponse::AuthoritativeBatch { batch })?;
                }
            }

            if session.subscribed.contains(&ViewerStream::Snapshot) {
                let snapshot = self.compat_snapshot();
                send_response(writer, &ViewerResponse::Snapshot { snapshot })?;
            }

            session.metrics = runtime_metrics(&self.world);
            if session.subscribed.contains(&ViewerStream::Metrics) {
                send_response(
                    writer,
                    &ViewerResponse::Metrics {
                        time: Some(self.world.state().time),
                        metrics: session.metrics.clone(),
                    },
                )?;
            }
        }

        if let Some(request_id) = request_id {
            let delta_logical_time = self
                .world
                .state()
                .time
                .saturating_sub(baseline_logical_time);
            let delta_event_seq =
                latest_runtime_event_seq(&self.world).saturating_sub(baseline_event_seq);
            let status = if delta_logical_time > 0 || delta_event_seq > 0 {
                ControlCompletionStatus::Advanced
            } else {
                ControlCompletionStatus::TimeoutNoProgress
            };
            let ack = ControlCompletionAck {
                request_id,
                status,
                delta_logical_time,
                delta_event_seq,
            };
            self.latest_player_gameplay_feedback = Some(player_gameplay_feedback_from_control_ack(
                &ViewerControl::Step { count: step_count },
                &ack,
            ));
            send_response(writer, &ViewerResponse::ControlCompletionAck { ack })?;
        }

        Ok(())
    }

    fn register_authoritative_batch(
        &mut self,
        events: &[WorldEvent],
    ) -> Result<AuthoritativeBatchFinality, ViewerRuntimeLiveServerError> {
        let commit_tick = self.world.state().time;
        let batch_id = format!(
            "{}-batch-{:020}",
            self.config.world_id, self.next_authoritative_batch_id
        );
        self.next_authoritative_batch_id = self.next_authoritative_batch_id.saturating_add(1);
        let state_root = compute_runtime_state_root(&self.world)?;
        let data_root = compute_batch_data_root(events)?;
        let tx_hash = compute_batch_tx_hash(
            batch_id.as_str(),
            state_root.as_str(),
            data_root.as_str(),
            commit_tick,
        )?;
        let record = RuntimeAuthoritativeBatchRecord {
            batch_id,
            tx_hash,
            commit_tick,
            confirm_height: commit_tick.saturating_add(AUTHORITATIVE_BATCH_CONFIRM_DELAY_TICKS),
            final_height: commit_tick
                .saturating_add(AUTHORITATIVE_BATCH_CONFIRM_DELAY_TICKS)
                .saturating_add(AUTHORITATIVE_BATCH_FINALITY_WINDOW_TICKS),
            state_root,
            data_root,
            event_seq_start: events.first().map(|event| event.id),
            event_seq_end: events.last().map(|event| event.id),
            finality_state: AuthoritativeFinalityState::Pending,
            challenge_state: RuntimeBatchChallengeState::None,
            active_challenge_id: None,
            events: events.to_vec(),
        };
        let response = record.as_wire(&self.settlement_ranking_gate);
        self.authoritative_batches.push_back(record);
        self.prune_authoritative_batch_history();
        Ok(response)
    }

    fn advance_authoritative_batch_finality(
        &mut self,
        current_tick: u64,
    ) -> Result<Vec<AuthoritativeBatchFinality>, ViewerRuntimeLiveServerError> {
        let mut changed_indexes = Vec::new();
        let mut newly_finalized_batch_ids = Vec::new();
        for index in 0..self.authoritative_batches.len() {
            let batch = self
                .authoritative_batches
                .get_mut(index)
                .expect("batch index is valid");
            if batch.challenge_state == RuntimeBatchChallengeState::Challenged {
                continue;
            }
            if batch.challenge_state == RuntimeBatchChallengeState::ResolvedFraudSlashed {
                continue;
            }
            if !batch.has_valid_commit_roots() {
                eprintln!(
                    "viewer runtime live: authoritative batch remains pending due missing/invalid roots batch_id={} state_root={} data_root={}",
                    batch.batch_id,
                    batch.state_root,
                    batch.data_root
                );
                continue;
            }
            let expected_data_root = compute_batch_data_root(batch.events.as_slice())?;
            if expected_data_root != batch.data_root {
                eprintln!(
                    "viewer runtime live: authoritative batch remains pending due data_root mismatch batch_id={} expected={} actual={}",
                    batch.batch_id,
                    expected_data_root,
                    batch.data_root
                );
                continue;
            }
            let expected_state = batch.expected_finality(current_tick);
            if expected_state > batch.finality_state {
                batch.finality_state = expected_state;
                if batch.finality_state == AuthoritativeFinalityState::Final {
                    newly_finalized_batch_ids.push(batch.batch_id.clone());
                }
                changed_indexes.push(index);
            }
        }

        for batch_id in newly_finalized_batch_ids {
            self.settlement_ranking_gate
                .promote_final(batch_id.as_str());
            self.capture_stable_checkpoint(batch_id.as_str())?;
        }

        let mut responses = Vec::new();
        for index in changed_indexes {
            if let Some(batch) = self.authoritative_batches.get(index) {
                responses.push(batch.as_wire(&self.settlement_ranking_gate));
            }
        }
        Ok(responses)
    }

    fn handle_authoritative_challenge(
        &mut self,
        command: AuthoritativeChallengeCommand,
    ) -> Result<
        (
            AuthoritativeChallengeAck<u64>,
            Option<AuthoritativeBatchFinality>,
        ),
        AuthoritativeChallengeError,
    > {
        match command {
            AuthoritativeChallengeCommand::Submit { request } => {
                self.submit_authoritative_challenge(request)
            }
            AuthoritativeChallengeCommand::Resolve { request } => {
                self.resolve_authoritative_challenge(request)
            }
        }
    }

    fn submit_authoritative_challenge(
        &mut self,
        request: AuthoritativeChallengeSubmitRequest,
    ) -> Result<
        (
            AuthoritativeChallengeAck<u64>,
            Option<AuthoritativeBatchFinality>,
        ),
        AuthoritativeChallengeError,
    > {
        if !is_valid_root_hash(request.recomputed_state_root.as_str()) {
            return Err(challenge_error(
                "invalid_recomputed_state_root",
                format!(
                    "recomputed_state_root must be 64 hex chars, got {}",
                    request.recomputed_state_root
                ),
                None,
                Some(request.batch_id.clone()),
            ));
        }
        if !is_valid_root_hash(request.recomputed_data_root.as_str()) {
            return Err(challenge_error(
                "invalid_recomputed_data_root",
                format!(
                    "recomputed_data_root must be 64 hex chars, got {}",
                    request.recomputed_data_root
                ),
                None,
                Some(request.batch_id.clone()),
            ));
        }

        let challenge_id = request.challenge_id.unwrap_or_else(|| {
            let generated = format!(
                "{}-challenge-{:020}",
                self.config.world_id, self.next_authoritative_challenge_id
            );
            self.next_authoritative_challenge_id =
                self.next_authoritative_challenge_id.saturating_add(1);
            generated
        });

        if let Some(existing) = self
            .authoritative_challenges
            .iter()
            .find(|record| record.challenge_id == challenge_id)
        {
            if existing.batch_id == request.batch_id
                && existing.watcher_id == request.watcher_id
                && existing.recomputed_state_root == request.recomputed_state_root
                && existing.recomputed_data_root == request.recomputed_data_root
            {
                let maybe_batch = self
                    .authoritative_batches
                    .iter()
                    .find(|batch| batch.batch_id == request.batch_id)
                    .map(|batch| batch.as_wire(&self.settlement_ranking_gate));
                return Ok((existing.as_ack(), maybe_batch));
            }
            return Err(challenge_error(
                "challenge_id_conflict",
                format!(
                    "challenge_id {} already exists with different payload",
                    challenge_id
                ),
                Some(challenge_id),
                Some(request.batch_id),
            ));
        }

        let current_tick = self.world.state().time;
        let Some(batch_index) = self
            .authoritative_batches
            .iter()
            .position(|batch| batch.batch_id == request.batch_id)
        else {
            return Err(challenge_error(
                "batch_not_found",
                format!("authoritative batch {} not found", request.batch_id),
                Some(challenge_id),
                Some(request.batch_id),
            ));
        };

        let batch = self
            .authoritative_batches
            .get_mut(batch_index)
            .expect("batch index is valid");
        if batch.finality_state == AuthoritativeFinalityState::Final
            || current_tick > batch.final_height
        {
            return Err(challenge_error(
                "challenge_window_closed",
                format!(
                    "challenge window closed for batch {} at tick={}",
                    batch.batch_id, current_tick
                ),
                Some(challenge_id),
                Some(batch.batch_id.clone()),
            ));
        }
        if batch.challenge_state == RuntimeBatchChallengeState::ResolvedFraudSlashed {
            return Err(challenge_error(
                "batch_already_slashed",
                format!("batch {} is already slashed", batch.batch_id),
                Some(challenge_id),
                Some(batch.batch_id.clone()),
            ));
        }
        if batch.challenge_state == RuntimeBatchChallengeState::Challenged {
            return Err(challenge_error(
                "batch_already_challenged",
                format!("batch {} already has an open challenge", batch.batch_id),
                Some(challenge_id),
                Some(batch.batch_id.clone()),
            ));
        }

        batch.challenge_state = RuntimeBatchChallengeState::Challenged;
        batch.active_challenge_id = Some(challenge_id.clone());
        let batch_wire = batch.as_wire(&self.settlement_ranking_gate);

        let record = RuntimeAuthoritativeChallengeRecord {
            challenge_id: challenge_id.clone(),
            batch_id: request.batch_id,
            watcher_id: request.watcher_id,
            recomputed_state_root: request.recomputed_state_root,
            recomputed_data_root: request.recomputed_data_root,
            status: AuthoritativeChallengeStatus::Challenged,
            submitted_at_tick: current_tick,
            resolved_at_tick: None,
            slash_applied: false,
            slash_reason: None,
        };
        let ack = record.as_ack();
        self.authoritative_challenges.push_back(record);
        self.prune_authoritative_challenge_history();
        Ok((ack, Some(batch_wire)))
    }

    fn resolve_authoritative_challenge(
        &mut self,
        request: AuthoritativeChallengeResolveRequest,
    ) -> Result<
        (
            AuthoritativeChallengeAck<u64>,
            Option<AuthoritativeBatchFinality>,
        ),
        AuthoritativeChallengeError,
    > {
        let current_tick = self.world.state().time;
        let Some(challenge_index) = self
            .authoritative_challenges
            .iter()
            .position(|record| record.challenge_id == request.challenge_id)
        else {
            return Err(challenge_error(
                "challenge_not_found",
                format!("challenge {} not found", request.challenge_id),
                Some(request.challenge_id),
                None,
            ));
        };

        let batch_id = self.authoritative_challenges[challenge_index]
            .batch_id
            .clone();
        let Some(batch_index) = self
            .authoritative_batches
            .iter()
            .position(|batch| batch.batch_id == batch_id)
        else {
            return Err(challenge_error(
                "batch_not_found",
                format!("authoritative batch {} not found", batch_id),
                Some(request.challenge_id),
                Some(batch_id),
            ));
        };

        let challenge = self
            .authoritative_challenges
            .get_mut(challenge_index)
            .expect("challenge index is valid");
        if challenge.status != AuthoritativeChallengeStatus::Challenged {
            return Err(challenge_error(
                "challenge_already_resolved",
                format!("challenge {} already resolved", challenge.challenge_id),
                Some(challenge.challenge_id.clone()),
                Some(challenge.batch_id.clone()),
            ));
        }

        let challenge_id = challenge.challenge_id.clone();
        let batch_id = challenge.batch_id.clone();
        let expected_state_root = challenge.recomputed_state_root.clone();
        let expected_data_root = challenge.recomputed_data_root.clone();

        let mut batch_wire = {
            let batch = self
                .authoritative_batches
                .get_mut(batch_index)
                .expect("batch index is valid");
            let state_root_match = expected_state_root == batch.state_root;
            let data_root_match = expected_data_root == batch.data_root;
            if state_root_match && data_root_match {
                challenge.status = AuthoritativeChallengeStatus::ResolvedNoFraud;
                challenge.resolved_at_tick = Some(current_tick);
                challenge.slash_applied = false;
                challenge.slash_reason = None;
                batch.challenge_state = RuntimeBatchChallengeState::ResolvedNoFraud;
                batch.active_challenge_id = None;
            } else {
                challenge.status = AuthoritativeChallengeStatus::ResolvedFraudSlashed;
                challenge.resolved_at_tick = Some(current_tick);
                challenge.slash_applied = true;
                let slash_reason = if !state_root_match && !data_root_match {
                    "state_root_and_data_root_mismatch"
                } else if !state_root_match {
                    "state_root_mismatch"
                } else {
                    "data_root_mismatch"
                };
                challenge.slash_reason = Some(slash_reason.to_string());
                batch.challenge_state = RuntimeBatchChallengeState::ResolvedFraudSlashed;
                batch.active_challenge_id = None;
            }
            batch.as_wire(&self.settlement_ranking_gate)
        };

        if self.authoritative_challenges[challenge_index].status
            == AuthoritativeChallengeStatus::ResolvedNoFraud
        {
            let updates = self
                .advance_authoritative_batch_finality(current_tick)
                .map_err(|err| {
                    challenge_error(
                        "resolve_failed",
                        format!("{err:?}"),
                        Some(challenge_id.clone()),
                        Some(batch_id.clone()),
                    )
                })?;
            if let Some(update) = updates
                .into_iter()
                .find(|update| update.batch_id == batch_id)
            {
                batch_wire = update;
            }
        }

        let ack = self.authoritative_challenges[challenge_index].as_ack();
        Ok((ack, Some(batch_wire)))
    }

    fn handle_authoritative_recovery(
        &mut self,
        command: AuthoritativeRecoveryCommand,
    ) -> Result<(AuthoritativeRecoveryAck<u64>, bool), AuthoritativeRecoveryError> {
        match command {
            AuthoritativeRecoveryCommand::RegisterSession { request } => {
                self.register_session_key(request).map(|ack| (ack, false))
            }
            AuthoritativeRecoveryCommand::Rollback { request } => self
                .rollback_to_stable_checkpoint(request)
                .map(|ack| (ack, true)),
            AuthoritativeRecoveryCommand::ReconnectSync { request } => {
                self.handle_reconnect_sync(request).map(|ack| (ack, false))
            }
            AuthoritativeRecoveryCommand::RevokeSession { request } => {
                self.revoke_session_key(request).map(|ack| (ack, false))
            }
            AuthoritativeRecoveryCommand::RotateSession { request } => {
                self.rotate_session_key(request).map(|ack| (ack, false))
            }
        }
    }

    fn rollback_to_stable_checkpoint(
        &mut self,
        request: AuthoritativeRollbackRequest,
    ) -> Result<AuthoritativeRecoveryAck<u64>, AuthoritativeRecoveryError> {
        let target_batch_id = request
            .target_batch_id
            .clone()
            .or_else(|| {
                self.stable_checkpoints
                    .back()
                    .map(|entry| entry.batch_id.clone())
            })
            .ok_or_else(|| {
                recovery_error(
                    "stable_checkpoint_not_found",
                    "no stable checkpoint available for rollback",
                    None,
                    None,
                    None,
                )
            })?;
        let checkpoint = self
            .stable_checkpoints
            .iter()
            .find(|entry| entry.batch_id == target_batch_id)
            .cloned()
            .ok_or_else(|| {
                recovery_error(
                    "stable_checkpoint_not_found",
                    format!("stable checkpoint for batch {} not found", target_batch_id),
                    Some(target_batch_id.clone()),
                    None,
                    None,
                )
            })?;
        let Some(batch_index) = self
            .authoritative_batches
            .iter()
            .position(|batch| batch.batch_id == target_batch_id)
        else {
            return Err(recovery_error(
                "batch_not_found",
                format!("authoritative batch {} not found", target_batch_id),
                Some(target_batch_id),
                None,
                None,
            ));
        };

        let reason = request.reason.trim();
        let rollback_reason = if reason.is_empty() {
            "authoritative_recovery_rollback".to_string()
        } else {
            reason.to_string()
        };
        self.world
            .rollback_to_snapshot_with_reconciliation(
                checkpoint.snapshot.clone(),
                checkpoint.journal.clone(),
                rollback_reason,
            )
            .map_err(|err| {
                recovery_error(
                    "rollback_failed",
                    format!("{err:?}"),
                    Some(checkpoint.batch_id.clone()),
                    None,
                    None,
                )
            })?;

        self.authoritative_batches
            .truncate(batch_index.saturating_add(1));
        self.authoritative_challenges.retain(|challenge| {
            self.authoritative_batches
                .iter()
                .any(|batch| batch.batch_id == challenge.batch_id)
        });
        self.prune_stable_checkpoints_after_batch(checkpoint.batch_id.as_str());
        self.rebuild_settlement_ranking_gate();
        self.reorg_epoch = self.reorg_epoch.saturating_add(1);

        let cursor = self.current_recovery_cursor().map_err(|err| {
            recovery_error(
                "cursor_compute_failed",
                format!("{err:?}"),
                Some(checkpoint.batch_id.clone()),
                None,
                None,
            )
        })?;
        Ok(AuthoritativeRecoveryAck {
            status: AuthoritativeRecoveryStatus::RolledBack,
            reorg_epoch: self.reorg_epoch,
            snapshot_height: cursor.snapshot_height,
            snapshot_hash: cursor.snapshot_hash,
            log_cursor: cursor.log_cursor,
            stable_batch_id: Some(checkpoint.batch_id),
            player_id: None,
            agent_id: None,
            session_pubkey: None,
            replaced_by_pubkey: None,
            session_epoch: None,
            message: Some("rollback applied to stable checkpoint".to_string()),
            acknowledged_at_tick: self.world.state().time,
        })
    }

    fn register_session_key(
        &mut self,
        request: AuthoritativeSessionRegisterRequest,
    ) -> Result<AuthoritativeRecoveryAck<u64>, AuthoritativeRecoveryError> {
        let Some(auth) = request.auth.as_ref() else {
            return Err(recovery_error(
                "auth_proof_required",
                "session_register requires auth proof",
                None,
                Some(request.player_id.clone()),
                request.public_key.clone(),
            ));
        };
        let verified = verify_session_register_auth_proof(&request, auth).map_err(|message| {
            recovery_error(
                control_plane::map_auth_verify_error_code(message.as_str()),
                message,
                None,
                Some(request.player_id.clone()),
                request.public_key.clone(),
            )
        })?;
        let session_epoch = self
            .session_policy
            .register_session(verified.player_id.as_str(), verified.public_key.as_str())
            .map_err(|message| {
                recovery_error(
                    map_session_policy_error_code(message.as_str()),
                    message,
                    None,
                    Some(verified.player_id.clone()),
                    Some(verified.public_key.clone()),
                )
            })?;
        self.llm_sidecar
            .consume_player_auth_nonce(verified.player_id.as_str(), verified.nonce)
            .map_err(|message| {
                recovery_error(
                    "auth_nonce_replay",
                    message,
                    None,
                    Some(verified.player_id.clone()),
                    Some(verified.public_key.clone()),
                )
            })?;

        let bound_agent_id = match request
            .requested_agent_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(agent_id) => {
                self.bind_player_session_agent(
                    agent_id,
                    verified.player_id.as_str(),
                    Some(verified.public_key.as_str()),
                    request.force_rebind,
                )
                .map_err(|message| {
                    recovery_error(
                        "player_bind_failed",
                        message,
                        None,
                        Some(verified.player_id.clone()),
                        Some(verified.public_key.clone()),
                    )
                })?;
                Some(agent_id.to_string())
            }
            None => self
                .llm_sidecar
                .bound_agent_for_player(verified.player_id.as_str())
                .map(ToOwned::to_owned),
        };

        let cursor = self.current_recovery_cursor().map_err(|err| {
            recovery_error(
                "cursor_compute_failed",
                format!("{err:?}"),
                None,
                Some(verified.player_id.clone()),
                Some(verified.public_key.clone()),
            )
        })?;
        Ok(AuthoritativeRecoveryAck {
            status: AuthoritativeRecoveryStatus::SessionRegistered,
            reorg_epoch: self.reorg_epoch,
            snapshot_height: cursor.snapshot_height,
            snapshot_hash: cursor.snapshot_hash,
            log_cursor: cursor.log_cursor,
            stable_batch_id: cursor.stable_batch_id,
            player_id: Some(verified.player_id),
            agent_id: bound_agent_id,
            session_pubkey: Some(verified.public_key),
            replaced_by_pubkey: None,
            session_epoch: Some(session_epoch),
            message: Some("session_registered".to_string()),
            acknowledged_at_tick: self.world.state().time,
        })
    }

    fn handle_reconnect_sync(
        &mut self,
        request: AuthoritativeReconnectSyncRequest,
    ) -> Result<AuthoritativeRecoveryAck<u64>, AuthoritativeRecoveryError> {
        let player_id = request.player_id.trim().to_string();
        if player_id.is_empty() {
            return Err(recovery_error(
                "player_id_required",
                "reconnect_sync requires non-empty player_id",
                None,
                None,
                None,
            ));
        }

        let session_pubkey = request
            .session_pubkey
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let mut session_epoch = None;
        if let Some(pubkey) = session_pubkey.as_deref() {
            let epoch = self
                .session_policy
                .validate_known_session_key(player_id.as_str(), pubkey)
                .map_err(|message| {
                    recovery_error(
                        map_session_policy_error_code(message.as_str()),
                        message,
                        None,
                        Some(player_id.clone()),
                        Some(pubkey.to_string()),
                    )
                })?;
            session_epoch = Some(epoch);
        }

        let cursor = self.current_recovery_cursor().map_err(|err| {
            recovery_error(
                "cursor_compute_failed",
                format!("{err:?}"),
                None,
                Some(player_id.clone()),
                session_pubkey.clone(),
            )
        })?;
        let stable_cursor = self
            .stable_checkpoints
            .back()
            .map(|entry| entry.log_cursor)
            .unwrap_or(0);

        let mut reasons = Vec::new();
        if request
            .expected_reorg_epoch
            .is_some_and(|epoch| epoch != self.reorg_epoch)
        {
            reasons.push(format!(
                "expected_reorg_epoch mismatch (client={}, server={})",
                request.expected_reorg_epoch.unwrap_or_default(),
                self.reorg_epoch
            ));
        }
        if let Some(last_known_cursor) = request.last_known_log_cursor {
            if last_known_cursor > cursor.log_cursor {
                reasons.push(format!(
                    "client cursor {} is ahead of server cursor {}",
                    last_known_cursor, cursor.log_cursor
                ));
            }
            if last_known_cursor < stable_cursor {
                reasons.push(format!(
                    "client cursor {} is behind stable cursor {}",
                    last_known_cursor, stable_cursor
                ));
            }
        }
        let message = if reasons.is_empty() {
            Some("delta_replay_allowed".to_string())
        } else {
            Some(format!("snapshot_reload_required: {}", reasons.join("; ")))
        };

        Ok(AuthoritativeRecoveryAck {
            status: AuthoritativeRecoveryStatus::CatchUpReady,
            reorg_epoch: self.reorg_epoch,
            snapshot_height: cursor.snapshot_height,
            snapshot_hash: cursor.snapshot_hash,
            log_cursor: cursor.log_cursor,
            stable_batch_id: cursor.stable_batch_id,
            player_id: Some(player_id),
            agent_id: self
                .llm_sidecar
                .bound_agent_for_player(request.player_id.as_str())
                .map(ToOwned::to_owned),
            session_pubkey,
            replaced_by_pubkey: None,
            session_epoch,
            message,
            acknowledged_at_tick: self.world.state().time,
        })
    }

    fn revoke_session_key(
        &mut self,
        request: AuthoritativeSessionRevokeRequest,
    ) -> Result<AuthoritativeRecoveryAck<u64>, AuthoritativeRecoveryError> {
        let player_id = request.player_id.trim().to_string();
        if player_id.is_empty() {
            return Err(recovery_error(
                "player_id_required",
                "revoke_session requires non-empty player_id",
                None,
                None,
                None,
            ));
        }

        let (revoked_pubkey, session_epoch) = self
            .session_policy
            .revoke_session(player_id.as_str(), request.session_pubkey.as_deref())
            .map_err(|message| {
                recovery_error(
                    map_session_policy_error_code(message.as_str()),
                    message,
                    None,
                    Some(player_id.clone()),
                    request.session_pubkey.clone(),
                )
            })?;
        self.clear_player_auth_runtime_state(player_id.as_str());
        self.apply_session_revoke_binding(player_id.as_str(), revoked_pubkey.as_str());

        let cursor = self.current_recovery_cursor().map_err(|err| {
            recovery_error(
                "cursor_compute_failed",
                format!("{err:?}"),
                None,
                Some(player_id.clone()),
                Some(revoked_pubkey.clone()),
            )
        })?;
        Ok(AuthoritativeRecoveryAck {
            status: AuthoritativeRecoveryStatus::SessionRevoked,
            reorg_epoch: self.reorg_epoch,
            snapshot_height: cursor.snapshot_height,
            snapshot_hash: cursor.snapshot_hash,
            log_cursor: cursor.log_cursor,
            stable_batch_id: cursor.stable_batch_id,
            player_id: Some(player_id),
            agent_id: None,
            session_pubkey: Some(revoked_pubkey),
            replaced_by_pubkey: None,
            session_epoch: Some(session_epoch),
            message: Some(request.revoke_reason.trim().to_string()),
            acknowledged_at_tick: self.world.state().time,
        })
    }

    fn rotate_session_key(
        &mut self,
        request: AuthoritativeSessionRotateRequest,
    ) -> Result<AuthoritativeRecoveryAck<u64>, AuthoritativeRecoveryError> {
        let player_id = request.player_id.trim().to_string();
        if player_id.is_empty() {
            return Err(recovery_error(
                "player_id_required",
                "rotate_session requires non-empty player_id",
                None,
                None,
                None,
            ));
        }

        let session_epoch = self
            .session_policy
            .rotate_session(
                player_id.as_str(),
                request.old_session_pubkey.as_str(),
                request.new_session_pubkey.as_str(),
            )
            .map_err(|message| {
                recovery_error(
                    map_session_policy_error_code(message.as_str()),
                    message,
                    None,
                    Some(player_id.clone()),
                    Some(request.old_session_pubkey.clone()),
                )
            })?;
        self.clear_player_auth_runtime_state(player_id.as_str());
        self.apply_session_rotate_binding(
            player_id.as_str(),
            request.old_session_pubkey.as_str(),
            request.new_session_pubkey.as_str(),
        );

        let cursor = self.current_recovery_cursor().map_err(|err| {
            recovery_error(
                "cursor_compute_failed",
                format!("{err:?}"),
                None,
                Some(player_id.clone()),
                Some(request.old_session_pubkey.clone()),
            )
        })?;
        Ok(AuthoritativeRecoveryAck {
            status: AuthoritativeRecoveryStatus::SessionRotated,
            reorg_epoch: self.reorg_epoch,
            snapshot_height: cursor.snapshot_height,
            snapshot_hash: cursor.snapshot_hash,
            log_cursor: cursor.log_cursor,
            stable_batch_id: cursor.stable_batch_id,
            player_id: Some(player_id),
            agent_id: self
                .llm_sidecar
                .bound_agent_for_player(request.player_id.as_str())
                .map(ToOwned::to_owned),
            session_pubkey: Some(request.old_session_pubkey),
            replaced_by_pubkey: Some(request.new_session_pubkey),
            session_epoch: Some(session_epoch),
            message: Some(request.rotate_reason.trim().to_string()),
            acknowledged_at_tick: self.world.state().time,
        })
    }

    fn current_recovery_cursor(
        &self,
    ) -> Result<RuntimeRecoveryCursor, ViewerRuntimeLiveServerError> {
        let snapshot_hash = compute_runtime_snapshot_hash(&self.world.snapshot())?;
        Ok(RuntimeRecoveryCursor {
            snapshot_hash,
            snapshot_height: self.world.state().time,
            log_cursor: latest_runtime_event_seq(&self.world),
            stable_batch_id: self
                .stable_checkpoints
                .back()
                .map(|entry| entry.batch_id.clone()),
        })
    }

    fn capture_stable_checkpoint(
        &mut self,
        batch_id: &str,
    ) -> Result<(), ViewerRuntimeLiveServerError> {
        let snapshot = self.world.snapshot();
        let journal = self.world.journal().clone();
        let checkpoint = RuntimeStableCheckpoint {
            batch_id: batch_id.to_string(),
            snapshot,
            journal,
            log_cursor: latest_runtime_event_seq(&self.world),
        };
        if let Some(index) = self
            .stable_checkpoints
            .iter()
            .position(|entry| entry.batch_id == batch_id)
        {
            let _ = self.stable_checkpoints.remove(index);
        }
        self.stable_checkpoints.push_back(checkpoint);
        self.prune_stable_checkpoint_history();
        Ok(())
    }

    fn prune_stable_checkpoint_history(&mut self) {
        while self.stable_checkpoints.len() > MAX_AUTHORITATIVE_STABLE_CHECKPOINTS {
            let _ = self.stable_checkpoints.pop_front();
        }
    }

    fn prune_stable_checkpoints_after_batch(&mut self, batch_id: &str) {
        if let Some(index) = self
            .stable_checkpoints
            .iter()
            .position(|entry| entry.batch_id == batch_id)
        {
            self.stable_checkpoints.truncate(index.saturating_add(1));
        }
    }

    fn rebuild_settlement_ranking_gate(&mut self) {
        let mut gate = RuntimeSettlementRankingGate::default();
        for batch in &self.authoritative_batches {
            if batch.finality_state == AuthoritativeFinalityState::Final
                && batch.challenge_state != RuntimeBatchChallengeState::ResolvedFraudSlashed
                && batch.challenge_state != RuntimeBatchChallengeState::Challenged
            {
                gate.promote_final(batch.batch_id.as_str());
            }
        }
        self.settlement_ranking_gate = gate;
    }

    fn clear_player_auth_runtime_state(&mut self, player_id: &str) {
        self.llm_sidecar
            .player_auth_last_nonce
            .remove(player_id.trim());
        self.llm_sidecar
            .clear_chat_intent_acks_for_player(player_id.trim());
    }

    fn apply_session_revoke_binding(&mut self, player_id: &str, _revoked_pubkey: &str) {
        if let Some(event) = self.llm_sidecar.clear_player_binding(player_id) {
            self.enqueue_virtual_event(event);
        }
    }

    fn apply_session_rotate_binding(
        &mut self,
        player_id: &str,
        old_pubkey: &str,
        new_pubkey: &str,
    ) {
        let mut affected_agents = Vec::new();
        for (agent_id, bound_player) in &self.llm_sidecar.agent_player_bindings {
            if bound_player == player_id {
                affected_agents.push(agent_id.clone());
            }
        }
        for agent_id in affected_agents {
            let should_replace = self
                .llm_sidecar
                .agent_public_key_bindings
                .get(agent_id.as_str())
                .map_or(true, |bound| bound == old_pubkey);
            if should_replace {
                self.llm_sidecar
                    .agent_public_key_bindings
                    .insert(agent_id, new_pubkey.to_string());
            }
        }
    }

    fn bind_player_session_agent(
        &mut self,
        agent_id: &str,
        player_id: &str,
        public_key: Option<&str>,
        allow_player_rebind: bool,
    ) -> Result<(), String> {
        control_plane::ensure_agent_player_access_runtime(
            &self.world,
            &self.llm_sidecar,
            agent_id,
            player_id,
            public_key,
        )
        .map_err(|err| err.message)?;
        for event in self.llm_sidecar.bind_agent_player(
            agent_id,
            player_id,
            public_key,
            allow_player_rebind,
        )? {
            self.enqueue_virtual_event(event);
        }
        Ok(())
    }

    fn emit_authoritative_batch_snapshot(
        &self,
        writer: &mut BufWriter<TcpStream>,
    ) -> Result<(), ViewerRuntimeLiveServerError> {
        for batch in &self.authoritative_batches {
            send_response(
                writer,
                &ViewerResponse::AuthoritativeBatch {
                    batch: batch.as_wire(&self.settlement_ranking_gate),
                },
            )?;
        }
        Ok(())
    }

    fn emit_authoritative_challenge_snapshot(
        &self,
        writer: &mut BufWriter<TcpStream>,
    ) -> Result<(), ViewerRuntimeLiveServerError> {
        for challenge in &self.authoritative_challenges {
            send_response(
                writer,
                &ViewerResponse::AuthoritativeChallengeAck {
                    ack: challenge.as_ack(),
                },
            )?;
        }
        Ok(())
    }

    fn prune_authoritative_batch_history(&mut self) {
        while self.authoritative_batches.len() > MAX_AUTHORITATIVE_BATCH_HISTORY {
            let Some(evicted) = self.authoritative_batches.pop_front() else {
                break;
            };
            self.settlement_ranking_gate
                .evict_batch(evicted.batch_id.as_str());
            self.authoritative_challenges
                .retain(|challenge| challenge.batch_id != evicted.batch_id);
            self.stable_checkpoints
                .retain(|entry| entry.batch_id != evicted.batch_id);
        }
    }

    fn prune_authoritative_challenge_history(&mut self) {
        while self.authoritative_challenges.len() > MAX_AUTHORITATIVE_CHALLENGE_HISTORY {
            let _ = self.authoritative_challenges.pop_front();
        }
    }

    fn compat_snapshot(&self) -> WorldSnapshot {
        let runtime_snapshot = self.world.snapshot();
        let runtime_journal_len = runtime_snapshot.journal_len;
        let next_event_id = runtime_snapshot.last_event_id.saturating_add(1).max(1);
        let next_action_id = runtime_snapshot.next_action_id.max(1);
        WorldSnapshot {
            version: SNAPSHOT_VERSION,
            chunk_generation_schema_version: CHUNK_GENERATION_SCHEMA_VERSION,
            time: self.world.state().time,
            config: self.snapshot_config.clone(),
            model: runtime_state_to_simulator_model(self.world.state(), &self.llm_sidecar),
            runtime_snapshot: Some(runtime_snapshot),
            player_gameplay: Some(build_player_gameplay_snapshot(
                self.world.state(),
                self.latest_player_gameplay_feedback.as_ref(),
                self.llm_sidecar.is_llm_mode() && self.llm_sidecar.supports_agent_chat(),
            )),
            chunk_runtime: ChunkRuntimeConfig::default(),
            next_event_id,
            next_action_id,
            pending_actions: Vec::new(),
            journal_len: runtime_journal_len,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct RuntimeLiveScript {
    phase: u8,
    move_direction: i64,
}

impl RuntimeLiveScript {
    fn enqueue(&mut self, world: &mut RuntimeWorld) {
        let mut agent_ids: Vec<String> = world.state().agents.keys().cloned().collect();
        agent_ids.sort();

        if agent_ids.is_empty() {
            world.submit_action(RuntimeAction::RegisterAgent {
                agent_id: "runtime-agent-0".to_string(),
                pos: GeoPos::new(0.0, 0.0, 0.0),
            });
            world.submit_action(RuntimeAction::RegisterAgent {
                agent_id: "runtime-agent-1".to_string(),
                pos: GeoPos::new(0.0, 0.0, 0.0),
            });
            return;
        }

        let phase = self.phase;
        self.phase = self.phase.wrapping_add(1) % 4;

        match phase {
            0 => {
                let first = &agent_ids[0];
                let Some(from_pos) = world.state().agents.get(first).map(|cell| cell.state.pos)
                else {
                    return;
                };
                if self.move_direction == 0 {
                    self.move_direction = 1;
                } else {
                    self.move_direction = -self.move_direction;
                }
                let delta_cm = (self.move_direction * 1_000) as f64;
                world.submit_action(RuntimeAction::MoveAgent {
                    agent_id: first.clone(),
                    to: GeoPos::new(from_pos.x_cm + delta_cm, from_pos.y_cm, from_pos.z_cm),
                });
            }
            1 => {
                if agent_ids.len() < 2 {
                    world.submit_action(RuntimeAction::MoveAgent {
                        agent_id: "missing-agent".to_string(),
                        to: GeoPos::new(0.0, 0.0, 0.0),
                    });
                    return;
                }
                let first = &agent_ids[0];
                let second = &agent_ids[1];
                let Some(target) = world.state().agents.get(first).map(|cell| cell.state.pos)
                else {
                    return;
                };
                world.submit_action(RuntimeAction::MoveAgent {
                    agent_id: second.clone(),
                    to: target,
                });
            }
            2 => {
                if agent_ids.len() < 2 {
                    world.submit_action(RuntimeAction::MoveAgent {
                        agent_id: "missing-agent".to_string(),
                        to: GeoPos::new(0.0, 0.0, 0.0),
                    });
                    return;
                }
                let from = &agent_ids[0];
                let to = &agent_ids[1];
                let _ = world.set_agent_resource_balance(from, ResourceKind::Electricity, 64);
                let _ = world.set_agent_resource_balance(to, ResourceKind::Electricity, 64);
                world.submit_action(RuntimeAction::EmitResourceTransfer {
                    from_agent_id: from.clone(),
                    to_agent_id: to.clone(),
                    kind: ResourceKind::Electricity,
                    amount: 1,
                });
            }
            _ => {
                world.submit_action(RuntimeAction::MoveAgent {
                    agent_id: "missing-agent".to_string(),
                    to: GeoPos::new(0.0, 0.0, 0.0),
                });
            }
        }
    }
}

struct RuntimeLiveSession {
    subscribed: HashSet<ViewerStream>,
    event_filters: Option<HashSet<ViewerEventKind>>,
    playing: bool,
    next_play_step_at: Option<Instant>,
    metrics: RunnerMetrics,
}

impl RuntimeLiveSession {
    fn new() -> Self {
        Self {
            subscribed: HashSet::new(),
            event_filters: None,
            playing: false,
            next_play_step_at: None,
            metrics: RunnerMetrics::default(),
        }
    }

    fn event_allowed(&self, event: &WorldEvent) -> bool {
        match &self.event_filters {
            Some(filters) => filters
                .iter()
                .any(|filter| viewer_event_kind_matches(filter, &event.kind)),
            None => true,
        }
    }

    fn should_advance_play_step(&mut self, interval: Duration) -> bool {
        if !self.playing {
            self.next_play_step_at = None;
            return false;
        }
        let now = Instant::now();
        if let Some(next_step_at) = self.next_play_step_at {
            if now < next_step_at {
                return false;
            }
        }
        self.next_play_step_at = Some(now + interval);
        true
    }
}

fn bootstrap_runtime_world(scenario: WorldScenario) -> Result<(RuntimeWorld, WorldConfig), String> {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(scenario, &config);
    let (model, _) = build_world_model(&config, &init)
        .map_err(|err| format!("runtime live bootstrap build_world_model failed: {err:?}"))?;

    let mut world = RuntimeWorld::new();
    world.set_resource_balance(ResourceKind::Electricity, 400);
    for (material, amount) in [
        ("structural_frame", 40),
        ("circuit_board", 4),
        ("servo_motor", 2),
        ("heat_coil", 6),
        ("refractory_brick", 8),
        ("iron_ore", 60),
        ("carbon_fuel", 20),
        ("copper_ore", 60),
        ("silicate_ore", 20),
        ("hardware_part", 40),
    ] {
        world
            .set_material_balance(material, amount)
            .map_err(|err| {
                format!(
                    "runtime live bootstrap set material balance failed material={} err={err:?}",
                    material
                )
            })?;
    }
    let mut seed_agents: Vec<(String, GeoPos, i64, i64)> = model
        .agents
        .iter()
        .map(|(agent_id, agent)| {
            (
                agent_id.clone(),
                agent.pos,
                agent.resources.get(ResourceKind::Electricity),
                agent.resources.get(ResourceKind::Data),
            )
        })
        .collect();
    seed_agents.sort_by(|left, right| left.0.cmp(&right.0));

    if seed_agents.is_empty() {
        seed_agents.push((
            "runtime-agent-0".to_string(),
            GeoPos::new(0.0, 0.0, 0.0),
            32,
            8,
        ));
        seed_agents.push((
            "runtime-agent-1".to_string(),
            GeoPos::new(0.0, 0.0, 0.0),
            32,
            8,
        ));
    }

    for (agent_id, pos, _, _) in &seed_agents {
        world.submit_action(RuntimeAction::RegisterAgent {
            agent_id: agent_id.clone(),
            pos: *pos,
        });
    }

    if world.pending_actions_len() > 0 {
        world
            .step()
            .map_err(|err| format!("runtime live bootstrap register step failed: {err:?}"))?;
    }

    for (agent_id, electricity, data) in world
        .state()
        .agents
        .keys()
        .cloned()
        .map(|agent_id| {
            let maybe_seed = seed_agents
                .iter()
                .find(|entry| entry.0 == agent_id)
                .cloned();
            match maybe_seed {
                Some((_, _, electricity, data)) => (agent_id, electricity.max(32), data.max(8)),
                None => (agent_id, 32, 8),
            }
        })
        .collect::<Vec<_>>()
    {
        world
            .set_agent_resource_balance(agent_id.as_str(), ResourceKind::Electricity, electricity)
            .map_err(|err| {
                format!(
                    "runtime live bootstrap set electricity failed agent={} err={err:?}",
                    agent_id
                )
            })?;
        world
            .set_agent_resource_balance(agent_id.as_str(), ResourceKind::Data, data)
            .map_err(|err| {
                format!(
                    "runtime live bootstrap set data failed agent={} err={err:?}",
                    agent_id
                )
            })?;
    }

    Ok((world, config))
}

fn runtime_metrics(world: &RuntimeWorld) -> RunnerMetrics {
    let total_ticks = world.state().time;
    let total_actions = world.journal().len() as u64;
    let action_rejected = world
        .journal()
        .events
        .iter()
        .filter(|event| {
            matches!(
                event.body,
                RuntimeWorldEventBody::Domain(RuntimeDomainEvent::ActionRejected { .. })
            )
        })
        .count() as u64;

    RunnerMetrics {
        total_ticks,
        total_agents: world.state().agents.len(),
        agents_active: world.state().agents.len(),
        agents_quota_exhausted: 0,
        total_actions,
        total_decisions: 0,
        actions_per_tick: if total_ticks > 0 {
            total_actions as f64 / total_ticks as f64
        } else {
            0.0
        },
        decisions_per_tick: 0.0,
        success_rate: if total_actions > 0 {
            (total_actions.saturating_sub(action_rejected)) as f64 / total_actions as f64
        } else {
            0.0
        },
        runtime_perf: Default::default(),
    }
}

fn latest_runtime_event_seq(world: &RuntimeWorld) -> u64 {
    world
        .journal()
        .events
        .last()
        .map(|event| event.id)
        .unwrap_or(0)
}

fn compute_runtime_state_root(
    world: &RuntimeWorld,
) -> Result<String, ViewerRuntimeLiveServerError> {
    let snapshot = world.snapshot();
    compute_runtime_snapshot_hash(&snapshot)
}

fn compute_runtime_snapshot_hash(
    snapshot: &RuntimeSnapshot,
) -> Result<String, ViewerRuntimeLiveServerError> {
    let bytes = serde_json::to_vec(snapshot).map_err(|err| {
        ViewerRuntimeLiveServerError::Serde(format!(
            "serialize runtime snapshot hash payload failed: {err}"
        ))
    })?;
    Ok(blake3_hex(bytes.as_slice()))
}

fn compute_batch_data_root(events: &[WorldEvent]) -> Result<String, ViewerRuntimeLiveServerError> {
    let bytes = serde_json::to_vec(events).map_err(|err| {
        ViewerRuntimeLiveServerError::Serde(format!(
            "serialize authoritative batch events for data_root failed: {err}"
        ))
    })?;
    Ok(blake3_hex(bytes.as_slice()))
}

fn compute_batch_tx_hash(
    batch_id: &str,
    state_root: &str,
    data_root: &str,
    commit_tick: u64,
) -> Result<String, ViewerRuntimeLiveServerError> {
    let payload = serde_json::json!({
        "batch_id": batch_id,
        "state_root": state_root,
        "data_root": data_root,
        "commit_tick": commit_tick,
    });
    let bytes = serde_json::to_vec(&payload).map_err(|err| {
        ViewerRuntimeLiveServerError::Serde(format!(
            "serialize authoritative batch tx payload failed: {err}"
        ))
    })?;
    Ok(blake3_hex(bytes.as_slice()))
}

fn is_valid_root_hash(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn challenge_error(
    code: impl Into<String>,
    message: impl Into<String>,
    challenge_id: Option<String>,
    batch_id: Option<String>,
) -> AuthoritativeChallengeError {
    AuthoritativeChallengeError {
        code: code.into(),
        message: message.into(),
        challenge_id,
        batch_id,
    }
}

fn recovery_error(
    code: impl Into<String>,
    message: impl Into<String>,
    batch_id: Option<String>,
    player_id: Option<String>,
    session_pubkey: Option<String>,
) -> AuthoritativeRecoveryError {
    AuthoritativeRecoveryError {
        code: code.into(),
        message: message.into(),
        batch_id,
        player_id,
        session_pubkey,
    }
}

fn map_session_policy_error_code(message: &str) -> &'static str {
    if message.contains("session_revoked") {
        return "session_revoked";
    }
    if message.contains("session_key_mismatch") {
        return "session_key_mismatch";
    }
    if message.contains("session_not_found") {
        return "session_not_found";
    }
    if message.contains("session_pubkey_invalid")
        || message.contains("session_player_id_invalid")
        || message.contains("session_rotation_invalid")
    {
        return "session_invalid";
    }
    "session_policy_error"
}

fn location_id_for_pos(pos: GeoPos) -> String {
    format!(
        "runtime:{}:{}:{}",
        pos.x_cm.round() as i64,
        pos.y_cm.round() as i64,
        pos.z_cm.round() as i64
    )
}

fn send_response(
    writer: &mut BufWriter<TcpStream>,
    response: &ViewerResponse,
) -> Result<(), ViewerRuntimeLiveServerError> {
    let payload = serde_json::to_string(response)
        .map_err(|err| ViewerRuntimeLiveServerError::Serde(err.to_string()))?;
    writer.write_all(payload.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn is_timeout_error(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
    )
}
