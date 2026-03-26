use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use super::auto_focus::focus_selection_with_transform;
use super::camera_controls::orbit_min_radius;
use super::selection_linking::apply_selection;
use super::*;

const AUTOMATION_STEPS_ENV: &str = "OASIS7_VIEWER_AUTOMATION_STEPS";
const AUTO_SELECT_ENV: &str = "OASIS7_VIEWER_AUTO_SELECT";
const AUTO_SELECT_TARGET_ENV: &str = "OASIS7_VIEWER_AUTO_SELECT_TARGET";
const VIEWER_PLAYER_ID_DEFAULT: &str = "viewer-player";
const VIEWER_PLAYER_ID_ENV: &str = "OASIS7_VIEWER_PLAYER_ID";
const VIEWER_AUTH_PUBLIC_KEY_ENV: &str = "OASIS7_VIEWER_AUTH_PUBLIC_KEY";
const VIEWER_AUTH_PRIVATE_KEY_ENV: &str = "OASIS7_VIEWER_AUTH_PRIVATE_KEY";
#[cfg(target_arch = "wasm32")]
const VIEWER_AUTH_BOOTSTRAP_OBJECT: &str = "__OASIS7_VIEWER_AUTH_ENV";

#[derive(Resource, Clone, Debug, PartialEq)]
pub(super) struct ViewerAutomationConfig {
    pub enabled: bool,
    pub steps: Vec<ViewerAutomationStep>,
}

impl Default for ViewerAutomationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            steps: Vec::new(),
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub(super) struct ViewerAutomationState {
    startup_step_index: usize,
    wait_until_secs: Option<f64>,
    runtime_steps: VecDeque<ViewerAutomationStep>,
    auth_nonce_floor: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ViewerAutomationStep {
    WaitSeconds(f64),
    SetMode(ViewerCameraMode),
    Focus(ViewerAutomationTarget),
    FocusSelection,
    Pan(Vec3),
    ZoomFactor(f32),
    OrbitDeg {
        yaw: f32,
        pitch: f32,
    },
    Select(ViewerAutomationTarget),
    PanelVisibility(ViewerAutomationVisibilityAction),
    TopPanelVisibility(ViewerAutomationVisibilityAction),
    ModuleVisibility {
        module: ViewerAutomationPanelModule,
        action: ViewerAutomationVisibilityAction,
    },
    TimelineSeek {
        tick: u64,
    },
    TimelineFilter {
        kind: crate::timeline_controls::TimelineMarkKindPublic,
        action: ViewerAutomationVisibilityAction,
    },
    TimelineJump {
        kind: crate::timeline_controls::TimelineMarkKindPublic,
    },
    SendAgentChat {
        agent_id: String,
        message: String,
    },
    ApplyPromptOverride {
        agent_id: String,
        field: ViewerAutomationPromptField,
        value: ViewerAutomationPromptValue,
    },
    SetLocale(ViewerAutomationLocaleAction),
    ApplyLayoutPreset(ViewerAutomationLayoutPreset),
    CycleMaterialVariant,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ViewerAutomationTarget {
    FirstKind(&'static str),
    KindId { kind: &'static str, id: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewerAutomationVisibilityAction {
    Show,
    Hide,
    Toggle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewerAutomationPanelModule {
    Controls,
    Overview,
    Chat,
    Overlay,
    Diagnosis,
    EventLink,
    Timeline,
    Details,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewerAutomationLocaleAction {
    Zh,
    En,
    Toggle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewerAutomationLayoutPreset {
    Mission,
    Command,
    Intel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewerAutomationPromptField {
    System,
    ShortTerm,
    LongTerm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ViewerAutomationPromptValue {
    Set(String),
    Clear,
}

const TARGET_KIND_AGENT: &str = "agent";
const TARGET_KIND_LOCATION: &str = "location";
const TARGET_KIND_ASSET: &str = "asset";
const TARGET_KIND_MODULE_VISUAL: &str = "module_visual";
const TARGET_KIND_POWER_PLANT: &str = "power_plant";
const TARGET_KIND_CHUNK: &str = "chunk";
const TARGET_KIND_FRAGMENT: &str = "fragment";
const POWER_FOCUS_RADIUS_SCALE_FROM_BASE: f32 = 3.0;
const POWER_FOCUS_RADIUS_MIN_M: f32 = 32.0;

struct TargetKindSpec<'a> {
    selection_kind: SelectionKind,
    entities: &'a HashMap<String, Entity>,
    first_filter: fn(&str) -> bool,
}

enum StepResult {
    Applied,
    AppliedYield,
    Pending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StepSource {
    StartupConfig,
    RuntimeQueue,
}

pub(super) fn viewer_automation_config_from_env() -> ViewerAutomationConfig {
    config_from_values(
        crate::viewer_env::viewer_env_var(AUTO_SELECT_ENV),
        crate::viewer_env::viewer_env_var(AUTO_SELECT_TARGET_ENV),
        crate::viewer_env::viewer_env_var(AUTOMATION_STEPS_ENV),
    )
}

pub(super) fn run_viewer_automation(
    time: Res<Time>,
    config: Res<ViewerAutomationConfig>,
    mut state: ResMut<ViewerAutomationState>,
    mut camera_mode: ResMut<ViewerCameraMode>,
    mut right_panel_layout: ResMut<RightPanelLayoutState>,
    mut module_visibility: ResMut<
        crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
    >,
    mut i18n: ResMut<crate::i18n::UiI18n>,
    mut variant_preview: ResMut<MaterialVariantPreviewState>,
    render_resources: (
        Res<Viewer3dConfig>,
        Option<Res<Viewer3dAssets>>,
        ResMut<Assets<StandardMaterial>>,
    ),
    runtime_resources: (
        Res<Viewer3dScene>,
        Option<Res<ViewerClient>>,
        Option<Res<ViewerState>>,
        Option<Res<ViewerControlProfileState>>,
    ),
    mut timeline: ResMut<crate::timeline_controls::TimelineUiState>,
    mut timeline_filters: Option<ResMut<crate::timeline_controls::TimelineMarkFilterState>>,
    mut selection: ResMut<ViewerSelection>,
    mut queries: ParamSet<(
        Query<(&mut OrbitCamera, &mut Transform, &mut Projection), With<Viewer3dCamera>>,
        Query<(&mut Transform, Option<&BaseScale>)>,
        Query<(&Transform, Option<&BaseScale>)>,
        Query<&LocationMarker>,
    )>,
) {
    let (viewer_config, assets, mut materials) = render_resources;
    let (scene, viewer_client, viewer_state, control_profile) = runtime_resources;
    let now = time.elapsed_secs_f64();
    if let Some(wait_until_secs) = state.wait_until_secs {
        if now < wait_until_secs {
            return;
        }
        state.wait_until_secs = None;
    }

    loop {
        let Some((source, step)) = next_step(&config, &state) else {
            return;
        };

        let result = apply_step(
            step,
            now,
            &scene,
            &viewer_config,
            &mut camera_mode,
            &mut right_panel_layout,
            &mut module_visibility,
            &mut i18n,
            &mut variant_preview,
            assets.as_deref(),
            &mut materials,
            viewer_client.as_deref(),
            viewer_state.as_deref(),
            control_profile.as_deref(),
            &mut timeline,
            timeline_filters.as_deref_mut(),
            &mut selection,
            &mut queries,
            &mut state,
        );
        match result {
            StepResult::Applied => {
                advance_step(&mut state, source);
                continue;
            }
            StepResult::AppliedYield => {
                advance_step(&mut state, source);
                return;
            }
            StepResult::Pending => return,
        }
    }
}

fn next_step(
    config: &ViewerAutomationConfig,
    state: &ViewerAutomationState,
) -> Option<(StepSource, ViewerAutomationStep)> {
    if let Some(step) = state.runtime_steps.front().cloned() {
        return Some((StepSource::RuntimeQueue, step));
    }
    if !config.enabled {
        return None;
    }
    config
        .steps
        .get(state.startup_step_index)
        .cloned()
        .map(|step| (StepSource::StartupConfig, step))
}

fn advance_step(state: &mut ViewerAutomationState, source: StepSource) {
    match source {
        StepSource::StartupConfig => {
            state.startup_step_index += 1;
        }
        StepSource::RuntimeQueue => {
            let _ = state.runtime_steps.pop_front();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn enqueue_runtime_steps(
    state: &mut ViewerAutomationState,
    steps: impl IntoIterator<Item = ViewerAutomationStep>,
) {
    state.runtime_steps.extend(steps);
}

#[cfg(target_arch = "wasm32")]
pub(super) fn parse_automation_steps(raw: &str) -> Vec<ViewerAutomationStep> {
    parse_steps(Some(raw))
}

#[cfg(target_arch = "wasm32")]
pub(super) fn parse_automation_mode(raw: &str) -> Option<ViewerCameraMode> {
    parse_mode(raw)
}

#[cfg(target_arch = "wasm32")]
pub(super) fn parse_automation_target(raw: &str) -> Option<ViewerAutomationTarget> {
    parse_target(raw)
}

fn apply_step(
    step: ViewerAutomationStep,
    now: f64,
    scene: &Viewer3dScene,
    viewer_config: &Viewer3dConfig,
    camera_mode: &mut ViewerCameraMode,
    right_panel_layout: &mut RightPanelLayoutState,
    module_visibility: &mut crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
    i18n: &mut crate::i18n::UiI18n,
    variant_preview: &mut MaterialVariantPreviewState,
    assets: Option<&Viewer3dAssets>,
    materials: &mut Assets<StandardMaterial>,
    viewer_client: Option<&ViewerClient>,
    viewer_state: Option<&ViewerState>,
    control_profile: Option<&ViewerControlProfileState>,
    timeline: &mut crate::timeline_controls::TimelineUiState,
    timeline_filters: Option<&mut crate::timeline_controls::TimelineMarkFilterState>,
    selection: &mut ViewerSelection,
    queries: &mut ParamSet<(
        Query<(&mut OrbitCamera, &mut Transform, &mut Projection), With<Viewer3dCamera>>,
        Query<(&mut Transform, Option<&BaseScale>)>,
        Query<(&Transform, Option<&BaseScale>)>,
        Query<&LocationMarker>,
    )>,
    state: &mut ViewerAutomationState,
) -> StepResult {
    match step {
        ViewerAutomationStep::WaitSeconds(seconds) => {
            state.wait_until_secs = Some(now + seconds.max(0.0));
            StepResult::AppliedYield
        }
        ViewerAutomationStep::SetMode(mode) => {
            if *camera_mode != mode {
                *camera_mode = mode;
                StepResult::AppliedYield
            } else {
                StepResult::Applied
            }
        }
        ViewerAutomationStep::Focus(target) => {
            let Some((entity, selection_kind, _)) = resolve_target_entity(scene, &target) else {
                return StepResult::Pending;
            };

            let (target_translation, target_base_scale) = {
                let transform_query = queries.p2();
                let Ok((target_transform, target_base_scale)) = transform_query.get(entity) else {
                    return StepResult::Pending;
                };
                (
                    target_transform.translation,
                    target_base_scale.map(|base_scale| base_scale.0),
                )
            };

            let mut camera_query = queries.p0();
            let Ok((mut orbit, mut camera_transform, _)) = camera_query.single_mut() else {
                return StepResult::Pending;
            };
            orbit.focus = target_translation;
            if let Some(target_radius) = automation_focus_radius_for_target(
                selection_kind,
                target_base_scale,
                viewer_config.effective_cm_to_unit(),
            ) {
                let min_radius = orbit_min_radius(viewer_config.effective_cm_to_unit());
                orbit.radius = target_radius.clamp(min_radius, ORBIT_MAX_RADIUS);
            }
            orbit.apply_to_transform(&mut camera_transform);
            StepResult::Applied
        }
        ViewerAutomationStep::FocusSelection => {
            let Some(current) = selection.current.clone() else {
                return StepResult::Applied;
            };

            let focus = {
                let transform_query = queries.p2();
                let Ok((target_transform, _)) = transform_query.get(current.entity) else {
                    return StepResult::Pending;
                };
                target_transform.translation
            };

            let mut camera_query = queries.p0();
            let Ok((mut orbit, mut camera_transform, mut projection)) = camera_query.single_mut()
            else {
                return StepResult::Pending;
            };

            focus_selection_with_transform(
                &current,
                focus,
                scene,
                viewer_config,
                camera_mode,
                &mut orbit,
                &mut camera_transform,
                &mut projection,
            );
            StepResult::Applied
        }
        ViewerAutomationStep::Pan(delta) => {
            let mut camera_query = queries.p0();
            let Ok((mut orbit, mut camera_transform, _)) = camera_query.single_mut() else {
                return StepResult::Pending;
            };
            orbit.focus += delta;
            orbit.apply_to_transform(&mut camera_transform);
            StepResult::Applied
        }
        ViewerAutomationStep::ZoomFactor(factor) => {
            let mut camera_query = queries.p0();
            let Ok((mut orbit, mut camera_transform, mut projection)) = camera_query.single_mut()
            else {
                return StepResult::Pending;
            };

            let min_radius = orbit_min_radius(viewer_config.effective_cm_to_unit());
            orbit.radius = (orbit.radius * factor.max(0.01)).clamp(min_radius, ORBIT_MAX_RADIUS);
            if *camera_mode == ViewerCameraMode::TwoD {
                if let Projection::Orthographic(ortho) = &mut *projection {
                    ortho.scale =
                        (ortho.scale * factor.max(0.01)).clamp(ORTHO_MIN_SCALE, ORTHO_MAX_SCALE);
                } else {
                    *projection = camera_projection_for_mode(ViewerCameraMode::TwoD, viewer_config);
                }
            }
            orbit.apply_to_transform(&mut camera_transform);
            StepResult::Applied
        }
        ViewerAutomationStep::OrbitDeg { yaw, pitch } => {
            if *camera_mode != ViewerCameraMode::ThreeD {
                return StepResult::Applied;
            }
            let mut camera_query = queries.p0();
            let Ok((mut orbit, mut camera_transform, _)) = camera_query.single_mut() else {
                return StepResult::Pending;
            };
            orbit.yaw += yaw.to_radians();
            orbit.pitch = (orbit.pitch + pitch.to_radians()).clamp(-1.54, 1.54);
            orbit.apply_to_transform(&mut camera_transform);
            StepResult::Applied
        }
        ViewerAutomationStep::Select(target) => {
            let Some((entity, kind, id)) = resolve_target_entity(scene, &target) else {
                return StepResult::Pending;
            };
            let name = if kind == SelectionKind::Location {
                queries
                    .p3()
                    .get(entity)
                    .ok()
                    .map(|marker| marker.name.clone())
            } else {
                None
            };

            apply_selection(
                selection,
                &mut queries.p1(),
                viewer_config,
                entity,
                kind,
                id,
                name,
            );
            StepResult::Applied
        }
        ViewerAutomationStep::PanelVisibility(action) => {
            let current_visible = !right_panel_layout.panel_hidden;
            let next_visible = apply_visibility_action(current_visible, action);
            right_panel_layout.panel_hidden = !next_visible;
            StepResult::Applied
        }
        ViewerAutomationStep::TopPanelVisibility(action) => {
            let current_visible = !right_panel_layout.top_panel_collapsed;
            let next_visible = apply_visibility_action(current_visible, action);
            right_panel_layout.top_panel_collapsed = !next_visible;
            StepResult::Applied
        }
        ViewerAutomationStep::ModuleVisibility { module, action } => {
            let flag = module_visibility_flag(module_visibility, module);
            let current_visible = *flag;
            *flag = apply_visibility_action(current_visible, action);
            StepResult::Applied
        }
        ViewerAutomationStep::TimelineSeek { tick } => {
            apply_timeline_seek_step(timeline, viewer_client, control_profile, tick);
            StepResult::Applied
        }
        ViewerAutomationStep::TimelineFilter { kind, action } => {
            if let Some(filters) = timeline_filters {
                let flag = timeline_filter_flag(filters, kind);
                *flag = apply_visibility_action(*flag, action);
            } else {
                bevy::log::warn!("viewer automation timeline filter step ignored: filters missing");
            }
            StepResult::Applied
        }
        ViewerAutomationStep::TimelineJump { kind } => {
            let Some(viewer_state) = viewer_state else {
                bevy::log::warn!(
                    "viewer automation timeline jump step ignored: viewer state missing"
                );
                return StepResult::Applied;
            };
            crate::timeline_controls::timeline_mark_jump_action(
                viewer_state,
                timeline,
                timeline_filters.as_deref(),
                kind,
            );
            StepResult::Applied
        }
        ViewerAutomationStep::SendAgentChat { agent_id, message } => {
            if let Err(err) =
                dispatch_agent_chat_step(viewer_client, state, agent_id.as_str(), message.as_str())
            {
                bevy::log::warn!("viewer automation chat step ignored: {err}");
            }
            StepResult::Applied
        }
        ViewerAutomationStep::ApplyPromptOverride {
            agent_id,
            field,
            value,
        } => {
            if let Err(err) = dispatch_prompt_override_step(
                viewer_client,
                state,
                agent_id.as_str(),
                field,
                &value,
            ) {
                bevy::log::warn!("viewer automation prompt step ignored: {err}");
            }
            StepResult::Applied
        }
        ViewerAutomationStep::SetLocale(action) => {
            i18n.locale = match action {
                ViewerAutomationLocaleAction::Zh => crate::i18n::UiLocale::ZhCn,
                ViewerAutomationLocaleAction::En => crate::i18n::UiLocale::EnUs,
                ViewerAutomationLocaleAction::Toggle => i18n.locale.toggled(),
            };
            StepResult::Applied
        }
        ViewerAutomationStep::ApplyLayoutPreset(preset) => {
            apply_layout_preset_automation(right_panel_layout, module_visibility, preset);
            StepResult::Applied
        }
        ViewerAutomationStep::CycleMaterialVariant => {
            variant_preview.active = variant_preview.active.next();
            if let Some(assets) = assets {
                apply_material_variant_to_scene_materials(
                    materials,
                    assets,
                    viewer_config,
                    variant_preview.active,
                );
            }
            StepResult::Applied
        }
    }
}

fn module_visibility_flag(
    state: &mut crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
    module: ViewerAutomationPanelModule,
) -> &mut bool {
    match module {
        ViewerAutomationPanelModule::Controls => &mut state.show_controls,
        ViewerAutomationPanelModule::Overview => &mut state.show_overview,
        ViewerAutomationPanelModule::Chat => &mut state.show_chat,
        ViewerAutomationPanelModule::Overlay => &mut state.show_overlay,
        ViewerAutomationPanelModule::Diagnosis => &mut state.show_diagnosis,
        ViewerAutomationPanelModule::EventLink => &mut state.show_event_link,
        ViewerAutomationPanelModule::Timeline => &mut state.show_timeline,
        ViewerAutomationPanelModule::Details => &mut state.show_details,
    }
}

fn apply_visibility_action(
    current_visible: bool,
    action: ViewerAutomationVisibilityAction,
) -> bool {
    match action {
        ViewerAutomationVisibilityAction::Show => true,
        ViewerAutomationVisibilityAction::Hide => false,
        ViewerAutomationVisibilityAction::Toggle => !current_visible,
    }
}

fn apply_layout_preset_automation(
    layout_state: &mut RightPanelLayoutState,
    module_visibility: &mut crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
    preset: ViewerAutomationLayoutPreset,
) {
    layout_state.panel_hidden = false;
    layout_state.top_panel_collapsed = false;
    module_visibility.show_controls = false;
    module_visibility.show_overlay = false;
    module_visibility.show_diagnosis = false;

    match preset {
        ViewerAutomationLayoutPreset::Mission => {
            module_visibility.show_overview = false;
            module_visibility.show_chat = false;
            module_visibility.show_event_link = false;
            module_visibility.show_timeline = false;
            module_visibility.show_details = false;
        }
        ViewerAutomationLayoutPreset::Command => {
            module_visibility.show_overview = false;
            module_visibility.show_chat = true;
            module_visibility.show_event_link = false;
            module_visibility.show_timeline = false;
            module_visibility.show_details = false;
        }
        ViewerAutomationLayoutPreset::Intel => {
            module_visibility.show_overview = true;
            module_visibility.show_chat = false;
            module_visibility.show_event_link = true;
            module_visibility.show_timeline = true;
            module_visibility.show_details = true;
        }
    }
}

fn apply_timeline_seek_step(
    timeline: &mut crate::timeline_controls::TimelineUiState,
    viewer_client: Option<&ViewerClient>,
    control_profile: Option<&ViewerControlProfileState>,
    tick: u64,
) {
    timeline.target_tick = tick;
    timeline.manual_override = true;
    timeline.drag_active = false;

    if let Some(client) = viewer_client {
        if !viewer_seek_supported(control_profile) {
            return;
        }
        let _ = dispatch_viewer_control(
            client,
            control_profile,
            oasis7::viewer::ViewerControl::Seek { tick },
            None,
        );
    }
}

fn timeline_filter_flag(
    filters: &mut crate::timeline_controls::TimelineMarkFilterState,
    kind: crate::timeline_controls::TimelineMarkKindPublic,
) -> &mut bool {
    match kind {
        crate::timeline_controls::TimelineMarkKindPublic::Error => &mut filters.show_error,
        crate::timeline_controls::TimelineMarkKindPublic::Llm => &mut filters.show_llm,
        crate::timeline_controls::TimelineMarkKindPublic::Peak => &mut filters.show_peak,
    }
}

#[derive(Clone, Debug)]
struct ViewerAutomationAuthSigner {
    player_id: String,
    public_key: String,
    private_key: String,
}

fn dispatch_agent_chat_step(
    viewer_client: Option<&ViewerClient>,
    automation_state: &mut ViewerAutomationState,
    agent_id: &str,
    message: &str,
) -> Result<(), String> {
    let client = viewer_client.ok_or_else(|| "viewer client unavailable".to_string())?;
    let signer = resolve_automation_auth_signer()?;
    let register_nonce = next_auth_nonce(automation_state);
    let mut session_register = oasis7::viewer::AuthoritativeSessionRegisterRequest {
        player_id: signer.player_id.clone(),
        public_key: Some(signer.public_key.clone()),
        auth: None,
        requested_agent_id: Some(agent_id.to_string()),
    };
    let register_proof = oasis7::viewer::sign_session_register_auth_proof(
        &session_register,
        register_nonce,
        signer.public_key.as_str(),
        signer.private_key.as_str(),
    )
    .map_err(|err| format!("sign session register failed: {err}"))?;
    session_register.auth = Some(register_proof);
    let nonce = next_auth_nonce(automation_state);
    let mut request = oasis7::viewer::AgentChatRequest {
        agent_id: agent_id.to_string(),
        message: message.to_string(),
        player_id: Some(signer.player_id.clone()),
        public_key: Some(signer.public_key.clone()),
        auth: None,
        intent_tick: None,
        intent_seq: Some(nonce),
    };
    let proof = oasis7::viewer::sign_agent_chat_auth_proof(
        &request,
        nonce,
        signer.public_key.as_str(),
        signer.private_key.as_str(),
    )
    .map_err(|err| format!("sign agent chat failed: {err}"))?;
    request.auth = Some(proof);
    client
        .tx
        .send(oasis7::viewer::ViewerRequest::AuthoritativeRecovery {
            command: oasis7::viewer::AuthoritativeRecoveryCommand::RegisterSession {
                request: session_register,
            },
        })
        .map_err(|err| format!("send session register failed: {err}"))?;
    client
        .tx
        .send(oasis7::viewer::ViewerRequest::AgentChat { request })
        .map_err(|err| format!("send agent chat failed: {err}"))
}

fn dispatch_prompt_override_step(
    viewer_client: Option<&ViewerClient>,
    automation_state: &mut ViewerAutomationState,
    agent_id: &str,
    field: ViewerAutomationPromptField,
    value: &ViewerAutomationPromptValue,
) -> Result<(), String> {
    let client = viewer_client.ok_or_else(|| "viewer client unavailable".to_string())?;
    let signer = resolve_automation_auth_signer()?;
    let register_nonce = next_auth_nonce(automation_state);
    let mut session_register = oasis7::viewer::AuthoritativeSessionRegisterRequest {
        player_id: signer.player_id.clone(),
        public_key: Some(signer.public_key.clone()),
        auth: None,
        requested_agent_id: Some(agent_id.to_string()),
    };
    let register_proof = oasis7::viewer::sign_session_register_auth_proof(
        &session_register,
        register_nonce,
        signer.public_key.as_str(),
        signer.private_key.as_str(),
    )
    .map_err(|err| format!("sign session register failed: {err}"))?;
    session_register.auth = Some(register_proof);
    let nonce = next_auth_nonce(automation_state);
    let mut request = oasis7::viewer::PromptControlApplyRequest {
        agent_id: agent_id.to_string(),
        player_id: signer.player_id.clone(),
        public_key: Some(signer.public_key.clone()),
        auth: None,
        expected_version: None,
        updated_by: Some(signer.player_id.clone()),
        system_prompt_override: None,
        short_term_goal_override: None,
        long_term_goal_override: None,
    };

    let patch = Some(prompt_override_patch(value));
    match field {
        ViewerAutomationPromptField::System => request.system_prompt_override = patch,
        ViewerAutomationPromptField::ShortTerm => request.short_term_goal_override = patch,
        ViewerAutomationPromptField::LongTerm => request.long_term_goal_override = patch,
    }

    let proof = oasis7::viewer::sign_prompt_control_apply_auth_proof(
        oasis7::viewer::PromptControlAuthIntent::Apply,
        &request,
        nonce,
        signer.public_key.as_str(),
        signer.private_key.as_str(),
    )
    .map_err(|err| format!("sign prompt apply failed: {err}"))?;
    request.auth = Some(proof);
    client
        .tx
        .send(oasis7::viewer::ViewerRequest::AuthoritativeRecovery {
            command: oasis7::viewer::AuthoritativeRecoveryCommand::RegisterSession {
                request: session_register,
            },
        })
        .map_err(|err| format!("send session register failed: {err}"))?;
    client
        .tx
        .send(oasis7::viewer::ViewerRequest::PromptControl {
            command: oasis7::viewer::PromptControlCommand::Apply { request },
        })
        .map_err(|err| format!("send prompt apply failed: {err}"))
}

fn prompt_override_patch(value: &ViewerAutomationPromptValue) -> Option<String> {
    match value {
        ViewerAutomationPromptValue::Set(text) => Some(text.clone()),
        ViewerAutomationPromptValue::Clear => None,
    }
}

fn resolve_automation_auth_signer() -> Result<ViewerAutomationAuthSigner, String> {
    let player_id = resolve_viewer_player_id()?;
    let public_key = resolve_required_auth_env(VIEWER_AUTH_PUBLIC_KEY_ENV)?;
    let private_key = resolve_required_auth_env(VIEWER_AUTH_PRIVATE_KEY_ENV)?;
    Ok(ViewerAutomationAuthSigner {
        player_id,
        public_key,
        private_key,
    })
}

fn resolve_viewer_player_id() -> Result<String, String> {
    match resolve_runtime_auth_value(VIEWER_PLAYER_ID_ENV) {
        Some(value) => Ok(value),
        None => Ok(VIEWER_PLAYER_ID_DEFAULT.to_string()),
    }
}

fn resolve_required_auth_env(key: &str) -> Result<String, String> {
    resolve_runtime_auth_value(key).ok_or_else(|| format!("{key} is not set"))
}

fn resolve_runtime_auth_value(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    if let Some(value) = resolve_wasm_auth_value(key) {
        return Some(value);
    }

    std::env::var(key)
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(target_arch = "wasm32")]
fn resolve_wasm_auth_value(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let store = js_sys::Reflect::get(
        window.as_ref(),
        &wasm_bindgen::JsValue::from_str(VIEWER_AUTH_BOOTSTRAP_OBJECT),
    )
    .ok()?;
    if store.is_null() || store.is_undefined() {
        return None;
    }
    js_sys::Reflect::get(&store, &wasm_bindgen::JsValue::from_str(key))
        .ok()?
        .as_string()
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn next_auth_nonce(state: &mut ViewerAutomationState) -> u64 {
    let nonce = state.auth_nonce_floor.max(1);
    state.auth_nonce_floor = nonce.saturating_add(1);
    nonce
}

fn resolve_target_entity(
    scene: &Viewer3dScene,
    target: &ViewerAutomationTarget,
) -> Option<(Entity, SelectionKind, String)> {
    match target {
        ViewerAutomationTarget::FirstKind(kind) => {
            let spec = target_kind_spec(scene, kind)?;
            let id = first_sorted_matching(spec.entities, spec.first_filter)
                .or_else(|| first_sorted_id(spec.entities))?;
            let entity = spec.entities.get(id.as_str()).copied()?;
            Some((entity, spec.selection_kind, id))
        }
        ViewerAutomationTarget::KindId { kind, id } => {
            let spec = target_kind_spec(scene, kind)?;
            spec.entities
                .get(id.as_str())
                .copied()
                .map(|entity| (entity, spec.selection_kind, id.clone()))
        }
    }
}

fn target_kind_spec<'a>(scene: &'a Viewer3dScene, kind: &str) -> Option<TargetKindSpec<'a>> {
    match kind {
        TARGET_KIND_AGENT => Some(TargetKindSpec {
            selection_kind: SelectionKind::Agent,
            entities: &scene.agent_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_LOCATION => Some(TargetKindSpec {
            selection_kind: SelectionKind::Location,
            entities: &scene.location_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_FRAGMENT => Some(TargetKindSpec {
            selection_kind: SelectionKind::Fragment,
            entities: &scene.location_entities,
            first_filter: is_fragment_id,
        }),
        TARGET_KIND_ASSET => Some(TargetKindSpec {
            selection_kind: SelectionKind::Asset,
            entities: &scene.asset_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_MODULE_VISUAL => Some(TargetKindSpec {
            selection_kind: SelectionKind::Asset,
            entities: &scene.module_visual_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_POWER_PLANT => Some(TargetKindSpec {
            selection_kind: SelectionKind::PowerPlant,
            entities: &scene.power_plant_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_CHUNK => Some(TargetKindSpec {
            selection_kind: SelectionKind::Chunk,
            entities: &scene.chunk_entities,
            first_filter: always_true,
        }),
        _ => None,
    }
}

fn first_sorted_id(items: &HashMap<String, Entity>) -> Option<String> {
    let mut ids: Vec<_> = items.keys().cloned().collect();
    ids.sort();
    ids.into_iter().next()
}

fn first_sorted_matching(
    items: &HashMap<String, Entity>,
    predicate: fn(&str) -> bool,
) -> Option<String> {
    let mut ids: Vec<_> = items
        .keys()
        .filter(|id| predicate(id.as_str()))
        .cloned()
        .collect();
    ids.sort();
    ids.into_iter().next()
}

fn always_true(_id: &str) -> bool {
    true
}

fn is_fragment_id(id: &str) -> bool {
    id.starts_with("frag-")
}

fn automation_focus_radius_for_target(
    selection_kind: SelectionKind,
    base_scale: Option<Vec3>,
    cm_to_unit: f32,
) -> Option<f32> {
    match selection_kind {
        SelectionKind::PowerPlant => {
            let units_per_meter = cm_to_unit.max(f32::EPSILON) * 100.0;
            let min_radius = POWER_FOCUS_RADIUS_MIN_M * units_per_meter;
            let scale_extent = base_scale
                .map(|scale| {
                    scale
                        .x
                        .abs()
                        .max(scale.y.abs())
                        .max(scale.z.abs())
                        .max(f32::EPSILON)
                })
                .unwrap_or(min_radius);
            Some((scale_extent * POWER_FOCUS_RADIUS_SCALE_FROM_BASE).max(min_radius))
        }
        _ => None,
    }
}

fn config_from_values(
    auto_select: Option<String>,
    auto_select_target: Option<String>,
    automation_steps: Option<String>,
) -> ViewerAutomationConfig {
    let steps = parse_steps(automation_steps.as_deref());
    if !steps.is_empty() {
        return ViewerAutomationConfig {
            enabled: true,
            steps,
        };
    }

    let target = auto_select_target
        .as_deref()
        .and_then(parse_target)
        .or_else(|| auto_select.as_deref().and_then(parse_target));
    let auto_select_enabled = auto_select
        .as_deref()
        .map(parse_truthy)
        .unwrap_or(auto_select_target.is_some());
    if auto_select_enabled {
        if let Some(target) = target {
            return ViewerAutomationConfig {
                enabled: true,
                steps: vec![ViewerAutomationStep::Select(target)],
            };
        }
    }

    ViewerAutomationConfig::default()
}

fn parse_steps(raw: Option<&str>) -> Vec<ViewerAutomationStep> {
    let mut steps = Vec::new();
    let Some(raw) = raw else {
        return steps;
    };

    for segment in raw.split(';') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let Some((key, value)) = segment.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        let parsed = match key.as_str() {
            "wait" => value
                .parse::<f64>()
                .ok()
                .map(ViewerAutomationStep::WaitSeconds),
            "mode" => parse_mode(value).map(ViewerAutomationStep::SetMode),
            "focus" => parse_focus_selection_step(value)
                .or_else(|| parse_target(value).map(ViewerAutomationStep::Focus)),
            "focus_selection" | "focus_selected" => parse_focus_selection_step(value),
            "pan" => parse_vec3(value).map(ViewerAutomationStep::Pan),
            "zoom" => value
                .parse::<f32>()
                .ok()
                .map(ViewerAutomationStep::ZoomFactor),
            "orbit" => parse_orbit(value),
            "select" => parse_target(value).map(ViewerAutomationStep::Select),
            "panel" => parse_visibility_action(value).map(ViewerAutomationStep::PanelVisibility),
            "top_panel" | "top" => {
                parse_visibility_action(value).map(ViewerAutomationStep::TopPanelVisibility)
            }
            "module" | "panel_module" | "module_visibility" => parse_module_visibility_step(value),
            "timeline_seek" | "seek_timeline" => parse_timeline_seek_step(value),
            "timeline_filter" | "timeline_mark_filter" => parse_timeline_filter_step(value),
            "timeline_jump" | "timeline_mark_jump" => parse_timeline_jump_step(value),
            "chat" | "chat_send" => parse_chat_step(value),
            "prompt_system" | "prompt_sys" => {
                parse_prompt_override_step(value, ViewerAutomationPromptField::System)
            }
            "prompt_short" | "prompt_short_term" | "prompt_stg" => {
                parse_prompt_override_step(value, ViewerAutomationPromptField::ShortTerm)
            }
            "prompt_long" | "prompt_long_term" | "prompt_ltg" => {
                parse_prompt_override_step(value, ViewerAutomationPromptField::LongTerm)
            }
            "locale" | "language" => {
                parse_locale_action(value).map(ViewerAutomationStep::SetLocale)
            }
            "layout" | "layout_preset" | "panel_layout" => {
                parse_layout_preset(value).map(ViewerAutomationStep::ApplyLayoutPreset)
            }
            "material_variant" | "variant" => parse_material_variant_step(value)
                .map(|_| ViewerAutomationStep::CycleMaterialVariant),
            _ => None,
        };
        if let Some(step) = parsed {
            steps.push(step);
        }
    }
    steps
}

fn parse_mode(raw: &str) -> Option<ViewerCameraMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "2d" | "two_d" | "twod" => Some(ViewerCameraMode::TwoD),
        "3d" | "three_d" | "threed" => Some(ViewerCameraMode::ThreeD),
        _ => None,
    }
}

fn parse_target(raw: &str) -> Option<ViewerAutomationTarget> {
    let value = raw.trim();
    if value.is_empty() {
        return None;
    }

    let normalized = value.to_ascii_lowercase();
    if let Some(kind_token) = normalized.strip_prefix("first_") {
        let kind = canonical_target_kind(kind_token)?;
        return Some(ViewerAutomationTarget::FirstKind(kind));
    }
    if let Some(kind_token) = normalized.strip_prefix("first:") {
        let kind = canonical_target_kind(kind_token)?;
        return Some(ViewerAutomationTarget::FirstKind(kind));
    }

    let (kind_token, id) = value.split_once(':')?;
    let kind = canonical_target_kind(kind_token)?;
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(ViewerAutomationTarget::KindId {
        kind,
        id: id.to_string(),
    })
}

fn canonical_target_kind(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "agent" => Some(TARGET_KIND_AGENT),
        "location" | "loc" => Some(TARGET_KIND_LOCATION),
        "fragment" | "frag" => Some(TARGET_KIND_FRAGMENT),
        "asset" => Some(TARGET_KIND_ASSET),
        "module_visual" | "module-visual" | "modulevisual" => Some(TARGET_KIND_MODULE_VISUAL),
        "power_plant" | "power-plant" | "powerplant" => Some(TARGET_KIND_POWER_PLANT),
        "chunk" => Some(TARGET_KIND_CHUNK),
        _ => None,
    }
}

fn parse_vec3(raw: &str) -> Option<Vec3> {
    let values: Vec<_> = raw
        .split(',')
        .map(|value| value.trim().parse::<f32>().ok())
        .collect();
    match values.as_slice() {
        [Some(x), Some(y), Some(z)] => Some(Vec3::new(*x, *y, *z)),
        _ => None,
    }
}

fn parse_orbit(raw: &str) -> Option<ViewerAutomationStep> {
    let values: Vec<_> = raw
        .split(',')
        .map(|value| value.trim().parse::<f32>().ok())
        .collect();
    match values.as_slice() {
        [Some(yaw), Some(pitch)] => Some(ViewerAutomationStep::OrbitDeg {
            yaw: *yaw,
            pitch: *pitch,
        }),
        _ => None,
    }
}

fn parse_focus_selection_step(raw: &str) -> Option<ViewerAutomationStep> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "selection" | "selected" | "current" | "current_selection" | "current-selection" | "1"
        | "true" | "yes" | "on" => Some(ViewerAutomationStep::FocusSelection),
        _ => None,
    }
}

fn parse_visibility_action(raw: &str) -> Option<ViewerAutomationVisibilityAction> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "show" | "visible" | "on" | "1" | "true" | "yes" => {
            Some(ViewerAutomationVisibilityAction::Show)
        }
        "hide" | "hidden" | "off" | "0" | "false" | "no" => {
            Some(ViewerAutomationVisibilityAction::Hide)
        }
        "toggle" | "switch" => Some(ViewerAutomationVisibilityAction::Toggle),
        _ => None,
    }
}

fn parse_panel_module(raw: &str) -> Option<ViewerAutomationPanelModule> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "controls" | "control" => Some(ViewerAutomationPanelModule::Controls),
        "overview" => Some(ViewerAutomationPanelModule::Overview),
        "chat" => Some(ViewerAutomationPanelModule::Chat),
        "overlay" => Some(ViewerAutomationPanelModule::Overlay),
        "diagnosis" | "diag" => Some(ViewerAutomationPanelModule::Diagnosis),
        "event_link" | "event-link" | "eventlink" => Some(ViewerAutomationPanelModule::EventLink),
        "timeline" => Some(ViewerAutomationPanelModule::Timeline),
        "details" | "detail" => Some(ViewerAutomationPanelModule::Details),
        _ => None,
    }
}

fn parse_module_visibility_step(raw: &str) -> Option<ViewerAutomationStep> {
    let (module_raw, action_raw) = raw.split_once(':')?;
    let module = parse_panel_module(module_raw)?;
    let action = parse_visibility_action(action_raw)?;
    Some(ViewerAutomationStep::ModuleVisibility { module, action })
}

fn parse_timeline_seek_step(raw: &str) -> Option<ViewerAutomationStep> {
    let tick = raw.trim().parse::<u64>().ok()?;
    Some(ViewerAutomationStep::TimelineSeek { tick })
}

fn parse_timeline_filter_step(raw: &str) -> Option<ViewerAutomationStep> {
    let (kind_raw, action_raw) = raw.split_once(':')?;
    let kind = parse_timeline_mark_kind(kind_raw)?;
    let action = parse_visibility_action(action_raw)?;
    Some(ViewerAutomationStep::TimelineFilter { kind, action })
}

fn parse_timeline_jump_step(raw: &str) -> Option<ViewerAutomationStep> {
    let kind = parse_timeline_mark_kind(raw)?;
    Some(ViewerAutomationStep::TimelineJump { kind })
}

fn parse_timeline_mark_kind(raw: &str) -> Option<crate::timeline_controls::TimelineMarkKindPublic> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "err" | "error" => Some(crate::timeline_controls::TimelineMarkKindPublic::Error),
        "llm" => Some(crate::timeline_controls::TimelineMarkKindPublic::Llm),
        "peak" | "resource_peak" | "resource-peak" | "resourcepeak" => {
            Some(crate::timeline_controls::TimelineMarkKindPublic::Peak)
        }
        _ => None,
    }
}

fn parse_chat_step(raw: &str) -> Option<ViewerAutomationStep> {
    let (agent_id, message) = parse_agent_and_text(raw)?;
    Some(ViewerAutomationStep::SendAgentChat { agent_id, message })
}

fn parse_prompt_override_step(
    raw: &str,
    field: ViewerAutomationPromptField,
) -> Option<ViewerAutomationStep> {
    let (agent_id, text_raw) = parse_agent_and_text(raw)?;
    let value = parse_prompt_value(text_raw.as_str())?;
    Some(ViewerAutomationStep::ApplyPromptOverride {
        agent_id,
        field,
        value,
    })
}

fn parse_agent_and_text(raw: &str) -> Option<(String, String)> {
    let (agent_id_raw, text_raw) = raw.split_once('|')?;
    let agent_id = agent_id_raw.trim();
    if agent_id.is_empty() {
        return None;
    }
    let text = decode_percent_text(text_raw)?;
    Some((agent_id.to_string(), text))
}

fn parse_prompt_value(raw: &str) -> Option<ViewerAutomationPromptValue> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "clear" | "none" | "null" | "default" => Some(ViewerAutomationPromptValue::Clear),
        _ => Some(ViewerAutomationPromptValue::Set(trimmed.to_string())),
    }
}

fn decode_percent_text(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let current = bytes[index];
        if current == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let high = from_hex_nibble(bytes[index + 1])?;
            let low = from_hex_nibble(bytes[index + 2])?;
            decoded.push((high << 4) | low);
            index += 3;
            continue;
        }
        if current == b'+' {
            decoded.push(b' ');
        } else {
            decoded.push(current);
        }
        index += 1;
    }
    let text = String::from_utf8(decoded).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn from_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn parse_material_variant_step(raw: &str) -> Option<()> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "next" | "cycle" | "toggle" | "f8" => Some(()),
        _ => None,
    }
}

fn parse_locale_action(raw: &str) -> Option<ViewerAutomationLocaleAction> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "zh" | "zh_cn" | "zh-cn" | "cn" | "chinese" => Some(ViewerAutomationLocaleAction::Zh),
        "en" | "en_us" | "en-us" | "english" => Some(ViewerAutomationLocaleAction::En),
        "toggle" | "switch" => Some(ViewerAutomationLocaleAction::Toggle),
        _ => None,
    }
}

fn parse_layout_preset(raw: &str) -> Option<ViewerAutomationLayoutPreset> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "mission" => Some(ViewerAutomationLayoutPreset::Mission),
        "command" => Some(ViewerAutomationLayoutPreset::Command),
        "intel" => Some(ViewerAutomationLayoutPreset::Intel),
        _ => None,
    }
}

fn parse_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_target_supports_first_and_explicit_variants() {
        assert_eq!(
            parse_target("first_agent"),
            Some(ViewerAutomationTarget::FirstKind(TARGET_KIND_AGENT))
        );
        assert_eq!(
            parse_target("first_location"),
            Some(ViewerAutomationTarget::FirstKind(TARGET_KIND_LOCATION))
        );
        assert_eq!(
            parse_target("first:power_plant"),
            Some(ViewerAutomationTarget::FirstKind(TARGET_KIND_POWER_PLANT))
        );
        assert_eq!(
            parse_target("agent:agent-1"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_AGENT,
                id: "agent-1".to_string(),
            })
        );
        assert_eq!(
            parse_target("location:loc-2"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_LOCATION,
                id: "loc-2".to_string(),
            })
        );
        assert_eq!(
            parse_target("power-plant:plant-1"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_POWER_PLANT,
                id: "plant-1".to_string(),
            })
        );
        assert_eq!(
            parse_target("modulevisual:mv-1"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_MODULE_VISUAL,
                id: "mv-1".to_string(),
            })
        );
        assert_eq!(
            parse_target("fragment:frag-2"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_FRAGMENT,
                id: "frag-2".to_string(),
            })
        );
        assert_eq!(
            parse_target("asset:a1"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_ASSET,
                id: "a1".to_string(),
            })
        );
        assert_eq!(parse_target("unknown:x"), None);
        assert_eq!(parse_target(""), None);
    }

    #[test]
    fn parse_steps_supports_camera_and_selection_actions() {
        let steps = parse_steps(Some(
            "mode=3d;wait=0.6;focus=agent:agent-0;pan=1,0,-2;zoom=0.8;orbit=10,-4;select=agent:agent-0",
        ));
        assert_eq!(
            steps,
            vec![
                ViewerAutomationStep::SetMode(ViewerCameraMode::ThreeD),
                ViewerAutomationStep::WaitSeconds(0.6),
                ViewerAutomationStep::Focus(ViewerAutomationTarget::KindId {
                    kind: TARGET_KIND_AGENT,
                    id: "agent-0".to_string(),
                }),
                ViewerAutomationStep::Pan(Vec3::new(1.0, 0.0, -2.0)),
                ViewerAutomationStep::ZoomFactor(0.8),
                ViewerAutomationStep::OrbitDeg {
                    yaw: 10.0,
                    pitch: -4.0
                },
                ViewerAutomationStep::Select(ViewerAutomationTarget::KindId {
                    kind: TARGET_KIND_AGENT,
                    id: "agent-0".to_string(),
                }),
            ]
        );
    }

    #[test]
    fn parse_steps_supports_panel_module_focus_selection_and_variant_actions() {
        let steps = parse_steps(Some(
            "panel=toggle;module=chat:hide;focus=selection;focus_selection=current;material_variant=next",
        ));
        assert_eq!(
            steps,
            vec![
                ViewerAutomationStep::PanelVisibility(ViewerAutomationVisibilityAction::Toggle),
                ViewerAutomationStep::ModuleVisibility {
                    module: ViewerAutomationPanelModule::Chat,
                    action: ViewerAutomationVisibilityAction::Hide,
                },
                ViewerAutomationStep::FocusSelection,
                ViewerAutomationStep::FocusSelection,
                ViewerAutomationStep::CycleMaterialVariant,
            ]
        );
    }

    #[test]
    fn parse_steps_supports_top_panel_locale_and_layout_actions() {
        let steps = parse_steps(Some(
            "top_panel=hide;locale=en;language=toggle;layout=command;top=show",
        ));
        assert_eq!(
            steps,
            vec![
                ViewerAutomationStep::TopPanelVisibility(ViewerAutomationVisibilityAction::Hide),
                ViewerAutomationStep::SetLocale(ViewerAutomationLocaleAction::En),
                ViewerAutomationStep::SetLocale(ViewerAutomationLocaleAction::Toggle),
                ViewerAutomationStep::ApplyLayoutPreset(ViewerAutomationLayoutPreset::Command),
                ViewerAutomationStep::TopPanelVisibility(ViewerAutomationVisibilityAction::Show),
            ]
        );
    }

    #[test]
    fn parse_steps_supports_timeline_seek_filter_and_jump_actions() {
        let steps = parse_steps(Some(
            "timeline_seek=42;timeline_filter=err:hide;timeline_filter=llm:toggle;timeline_jump=peak",
        ));
        assert_eq!(
            steps,
            vec![
                ViewerAutomationStep::TimelineSeek { tick: 42 },
                ViewerAutomationStep::TimelineFilter {
                    kind: crate::timeline_controls::TimelineMarkKindPublic::Error,
                    action: ViewerAutomationVisibilityAction::Hide,
                },
                ViewerAutomationStep::TimelineFilter {
                    kind: crate::timeline_controls::TimelineMarkKindPublic::Llm,
                    action: ViewerAutomationVisibilityAction::Toggle,
                },
                ViewerAutomationStep::TimelineJump {
                    kind: crate::timeline_controls::TimelineMarkKindPublic::Peak,
                },
            ]
        );
    }

    #[test]
    fn parse_steps_supports_chat_and_prompt_actions() {
        let steps = parse_steps(Some(
            "chat=agent-1|hello+world%21;prompt_system=agent-1|clear;prompt_short=agent-1|Need%20power%20first",
        ));
        assert_eq!(
            steps,
            vec![
                ViewerAutomationStep::SendAgentChat {
                    agent_id: "agent-1".to_string(),
                    message: "hello world!".to_string(),
                },
                ViewerAutomationStep::ApplyPromptOverride {
                    agent_id: "agent-1".to_string(),
                    field: ViewerAutomationPromptField::System,
                    value: ViewerAutomationPromptValue::Clear,
                },
                ViewerAutomationStep::ApplyPromptOverride {
                    agent_id: "agent-1".to_string(),
                    field: ViewerAutomationPromptField::ShortTerm,
                    value: ViewerAutomationPromptValue::Set("Need power first".to_string()),
                },
            ]
        );
    }

    #[test]
    fn parse_steps_ignores_invalid_module_and_variant_actions() {
        let steps = parse_steps(Some(
            "module=chat;module=unknown:show;module=timeline:toggle;material_variant=bad;variant=cycle",
        ));
        assert_eq!(
            steps,
            vec![
                ViewerAutomationStep::ModuleVisibility {
                    module: ViewerAutomationPanelModule::Timeline,
                    action: ViewerAutomationVisibilityAction::Toggle,
                },
                ViewerAutomationStep::CycleMaterialVariant,
            ]
        );
    }

    #[test]
    fn parse_steps_ignores_invalid_locale_and_layout_actions() {
        let steps = parse_steps(Some("locale=jp;language=english;layout=unknown"));
        assert_eq!(
            steps,
            vec![ViewerAutomationStep::SetLocale(
                ViewerAutomationLocaleAction::En
            )]
        );
    }

    #[test]
    fn parse_steps_ignores_invalid_timeline_actions() {
        let steps = parse_steps(Some(
            "timeline_seek=-1;timeline_seek=abc;timeline_filter=foo:show;timeline_filter=err:unknown;timeline_jump=other;timeline_mark_jump=error",
        ));
        assert_eq!(
            steps,
            vec![ViewerAutomationStep::TimelineJump {
                kind: crate::timeline_controls::TimelineMarkKindPublic::Error,
            }]
        );
    }

    #[test]
    fn parse_steps_ignores_invalid_chat_and_prompt_actions() {
        let steps = parse_steps(Some(
            "chat=agent-1;chat=agent-2|%ZZ;prompt_system=agent-1|;prompt_long=|hello;prompt_short=agent-3|default",
        ));
        assert_eq!(
            steps,
            vec![ViewerAutomationStep::ApplyPromptOverride {
                agent_id: "agent-3".to_string(),
                field: ViewerAutomationPromptField::ShortTerm,
                value: ViewerAutomationPromptValue::Clear,
            }]
        );
    }

    #[test]
    fn apply_visibility_action_respects_show_hide_toggle() {
        assert!(apply_visibility_action(
            false,
            ViewerAutomationVisibilityAction::Show
        ));
        assert!(!apply_visibility_action(
            true,
            ViewerAutomationVisibilityAction::Hide
        ));
        assert!(!apply_visibility_action(
            true,
            ViewerAutomationVisibilityAction::Toggle
        ));
        assert!(apply_visibility_action(
            false,
            ViewerAutomationVisibilityAction::Toggle
        ));
    }

    #[test]
    fn apply_layout_preset_automation_updates_panel_and_module_visibility() {
        let mut layout_state = RightPanelLayoutState {
            top_panel_collapsed: true,
            panel_hidden: true,
        };
        let mut module_visibility =
            crate::right_panel_module_visibility::RightPanelModuleVisibilityState::default();

        apply_layout_preset_automation(
            &mut layout_state,
            &mut module_visibility,
            ViewerAutomationLayoutPreset::Intel,
        );
        assert!(!layout_state.panel_hidden);
        assert!(!layout_state.top_panel_collapsed);
        assert!(!module_visibility.show_controls);
        assert!(module_visibility.show_overview);
        assert!(!module_visibility.show_chat);
        assert!(module_visibility.show_event_link);
        assert!(module_visibility.show_timeline);
        assert!(module_visibility.show_details);
    }

    #[test]
    fn apply_layout_preset_automation_command_keeps_player_surface_compact() {
        let mut layout_state = RightPanelLayoutState {
            top_panel_collapsed: true,
            panel_hidden: true,
        };
        let mut module_visibility =
            crate::right_panel_module_visibility::RightPanelModuleVisibilityState::default();

        apply_layout_preset_automation(
            &mut layout_state,
            &mut module_visibility,
            ViewerAutomationLayoutPreset::Command,
        );

        assert!(!layout_state.panel_hidden);
        assert!(!layout_state.top_panel_collapsed);
        assert!(!module_visibility.show_controls);
        assert!(!module_visibility.show_overview);
        assert!(module_visibility.show_chat);
        assert!(!module_visibility.show_event_link);
        assert!(!module_visibility.show_timeline);
        assert!(!module_visibility.show_details);
    }

    #[test]
    fn config_from_values_uses_auto_select_when_steps_absent() {
        let config = config_from_values(
            Some("1".to_string()),
            Some("agent:agent-2".to_string()),
            None,
        );
        assert!(config.enabled);
        assert_eq!(
            config.steps,
            vec![ViewerAutomationStep::Select(
                ViewerAutomationTarget::KindId {
                    kind: TARGET_KIND_AGENT,
                    id: "agent-2".to_string(),
                }
            )]
        );
    }

    #[test]
    fn config_from_values_prioritizes_explicit_steps() {
        let config = config_from_values(
            Some("1".to_string()),
            Some("agent:agent-2".to_string()),
            Some("wait=0.2;select=first_agent".to_string()),
        );
        assert!(config.enabled);
        assert_eq!(
            config.steps,
            vec![
                ViewerAutomationStep::WaitSeconds(0.2),
                ViewerAutomationStep::Select(ViewerAutomationTarget::FirstKind(TARGET_KIND_AGENT)),
            ]
        );
    }

    #[test]
    fn resolve_target_entity_supports_extended_scene_kinds() {
        let mut scene = Viewer3dScene::default();
        scene
            .agent_entities
            .insert("agent-1".to_string(), Entity::from_bits(1));
        scene
            .location_entities
            .insert("loc-1".to_string(), Entity::from_bits(2));
        scene
            .location_entities
            .insert("frag-2".to_string(), Entity::from_bits(3));
        scene
            .asset_entities
            .insert("asset-1".to_string(), Entity::from_bits(4));
        scene
            .module_visual_entities
            .insert("mv-1".to_string(), Entity::from_bits(5));
        scene
            .power_plant_entities
            .insert("plant-1".to_string(), Entity::from_bits(6));
        scene
            .chunk_entities
            .insert("chunk-1".to_string(), Entity::from_bits(7));

        let fragment_target = ViewerAutomationTarget::FirstKind(TARGET_KIND_FRAGMENT);
        let Some((fragment_entity, fragment_kind, fragment_id)) =
            resolve_target_entity(&scene, &fragment_target)
        else {
            panic!("fragment target should resolve");
        };
        assert_eq!(fragment_entity, Entity::from_bits(3));
        assert_eq!(fragment_kind, SelectionKind::Fragment);
        assert_eq!(fragment_id, "frag-2");

        let module_target = ViewerAutomationTarget::KindId {
            kind: TARGET_KIND_MODULE_VISUAL,
            id: "mv-1".to_string(),
        };
        let Some((module_entity, module_kind, module_id)) =
            resolve_target_entity(&scene, &module_target)
        else {
            panic!("module_visual target should resolve");
        };
        assert_eq!(module_entity, Entity::from_bits(5));
        assert_eq!(module_kind, SelectionKind::Asset);
        assert_eq!(module_id, "mv-1");

        let chunk_target = ViewerAutomationTarget::KindId {
            kind: TARGET_KIND_CHUNK,
            id: "chunk-1".to_string(),
        };
        let Some((chunk_entity, chunk_kind, chunk_id)) =
            resolve_target_entity(&scene, &chunk_target)
        else {
            panic!("chunk target should resolve");
        };
        assert_eq!(chunk_entity, Entity::from_bits(7));
        assert_eq!(chunk_kind, SelectionKind::Chunk);
        assert_eq!(chunk_id, "chunk-1");
    }

    #[test]
    fn automation_focus_radius_for_power_target_has_reasonable_floor() {
        let cm_to_unit = 0.0000384;
        let radius = automation_focus_radius_for_target(
            SelectionKind::PowerPlant,
            Some(Vec3::splat(0.012)),
            cm_to_unit,
        )
        .expect("power target radius should resolve");
        let min_floor = POWER_FOCUS_RADIUS_MIN_M * cm_to_unit * 100.0;
        assert!(radius >= min_floor);
        assert!(radius > 0.0);
    }

    #[test]
    fn automation_focus_radius_for_non_power_target_is_none() {
        assert_eq!(
            automation_focus_radius_for_target(
                SelectionKind::Location,
                Some(Vec3::splat(0.5)),
                0.0000384
            ),
            None
        );
    }
}
