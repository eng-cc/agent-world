#[cfg(target_arch = "wasm32")]
use crate::viewer_automation::{
    enqueue_runtime_steps, parse_automation_mode, parse_automation_steps, parse_automation_target,
};
#[cfg(target_arch = "wasm32")]
use crate::{
    dispatch_viewer_control, ViewerAutomationState, ViewerClient, ViewerControlProfileState,
    ViewerSelection, ViewerState,
};
#[cfg(target_arch = "wasm32")]
use crate::{ConnectionStatus, SelectionKind};
use crate::{OrbitCamera, Viewer3dCamera, ViewerCameraMode};
#[cfg(not(target_arch = "wasm32"))]
use crate::{
    ViewerAutomationState, ViewerClient, ViewerControlProfileState, ViewerSelection, ViewerState,
};
#[cfg(target_arch = "wasm32")]
use agent_world::viewer::{ViewerControl, ViewerRequest};
use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::collections::VecDeque;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys::{Array, Object, Reflect as JsReflect};
#[cfg(target_arch = "wasm32")]
use web_sys::UrlSearchParams;
#[cfg(target_arch = "wasm32")]
const TEST_API_QUERY_KEY: &str = "test_api";
#[cfg(target_arch = "wasm32")]
const TEST_API_GLOBAL_NAME: &str = "__AW_TEST__";
#[cfg(target_arch = "wasm32")]
const WEB_TEST_API_CONTROL_ACTIONS: [&str; 3] = ["play", "pause", "step"];
#[cfg(target_arch = "wasm32")]
const CONTROL_STALL_FRAME_THRESHOLD: u32 = 150;
#[cfg(target_arch = "wasm32")]
enum WebTestApiCommand {
    EnqueueSteps(Vec<crate::viewer_automation::ViewerAutomationStep>),
    SendControl {
        control: ViewerControl,
        feedback_id: u64,
    },
}
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
struct WebTestApiControlFeedback {
    id: u64,
    action: String,
    accepted: bool,
    enqueued: bool,
    stage: String,
    parsed_control: Option<String>,
    reason: Option<String>,
    hint: Option<String>,
    effect: String,
    baseline_logical_time: u64,
    baseline_event_seq: u64,
    delta_logical_time: u64,
    delta_event_seq: u64,
    awaiting_effect: bool,
    no_progress_frames: u32,
}
#[derive(Clone, Debug)]
pub(super) struct WebTestApiControlFeedbackSnapshot {
    pub(super) action: String,
    pub(super) stage: String,
    pub(super) reason: Option<String>,
    pub(super) hint: Option<String>,
    pub(super) effect: String,
}
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug)]
struct WebTestApiStateSnapshot {
    connection_status: &'static str,
    logical_time: u64,
    event_seq: u64,
    selected_kind: Option<String>,
    selected_id: Option<String>,
    error_count: u64,
    last_error: Option<String>,
    event_count: usize,
    trace_count: usize,
    camera_mode: &'static str,
    camera_radius: f64,
    camera_ortho_scale: f64,
    last_control_feedback: Option<WebTestApiControlFeedback>,
}
#[cfg(target_arch = "wasm32")]
impl Default for WebTestApiStateSnapshot {
    fn default() -> Self {
        Self {
            connection_status: "connecting",
            logical_time: 0,
            event_seq: 0,
            selected_kind: None,
            selected_id: None,
            error_count: 0,
            last_error: None,
            event_count: 0,
            trace_count: 0,
            camera_mode: "3d",
            camera_radius: 0.0,
            camera_ortho_scale: 0.0,
            last_control_feedback: None,
        }
    }
}
#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_TEST_API_COMMAND_QUEUE: RefCell<VecDeque<WebTestApiCommand>> = RefCell::new(VecDeque::new());
    static WEB_TEST_API_STATE_SNAPSHOT: RefCell<WebTestApiStateSnapshot> = RefCell::new(WebTestApiStateSnapshot::default());
    static WEB_TEST_API_CONTROL_FEEDBACK_ID: RefCell<u64> = const { RefCell::new(0) };
}
#[cfg(target_arch = "wasm32")]
pub(super) struct WebTestApiBindings {
    _api: Object,
    _run_steps: Closure<dyn FnMut(JsValue)>,
    _set_mode: Closure<dyn FnMut(JsValue)>,
    _focus: Closure<dyn FnMut(JsValue)>,
    _select: Closure<dyn FnMut(JsValue)>,
    _describe_controls: Closure<dyn FnMut() -> JsValue>,
    _fill_control_example: Closure<dyn FnMut(JsValue) -> JsValue>,
    _send_control: Closure<dyn FnMut(JsValue, JsValue) -> JsValue>,
    _get_state: Closure<dyn FnMut() -> JsValue>,
}

#[cfg(target_arch = "wasm32")]
fn log_api_warning(message: &str) {
    web_sys::console::warn_1(&JsValue::from_str(message));
}

#[cfg(target_arch = "wasm32")]
fn web_test_api_enabled(window: &web_sys::Window) -> bool {
    if cfg!(debug_assertions) {
        return true;
    }
    let Ok(search) = window.location().search() else {
        return false;
    };
    let Ok(params) = UrlSearchParams::new_with_str(&search) else {
        return false;
    };
    params
        .get(TEST_API_QUERY_KEY)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(target_arch = "wasm32")]
fn push_command(command: WebTestApiCommand) {
    WEB_TEST_API_COMMAND_QUEUE.with(|queue| {
        queue.borrow_mut().push_back(command);
    });
}

#[cfg(target_arch = "wasm32")]
fn next_control_feedback_id() -> u64 {
    WEB_TEST_API_CONTROL_FEEDBACK_ID.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter = counter.saturating_add(1);
        *counter
    })
}

#[cfg(target_arch = "wasm32")]
fn latest_logical_time_event_seq() -> (u64, u64) {
    WEB_TEST_API_STATE_SNAPSHOT.with(|slot| {
        let snapshot = slot.borrow();
        (snapshot.logical_time, snapshot.event_seq)
    })
}

#[cfg(target_arch = "wasm32")]
fn control_payload_example(action: &str) -> JsValue {
    match action {
        "play" | "pause" => JsValue::NULL,
        "step" => {
            let payload = Object::new();
            let _ = JsReflect::set(
                &payload,
                &JsValue::from_str("count"),
                &JsValue::from_f64(5.0),
            );
            JsValue::from(payload)
        }
        _ => JsValue::NULL,
    }
}

#[cfg(target_arch = "wasm32")]
fn control_description(action: &str, is_zh: bool) -> &'static str {
    match (action, is_zh) {
        ("play", true) => "开始连续推进世界",
        ("play", false) => "Start continuous world advancement",
        ("pause", true) => "暂停连续推进",
        ("pause", false) => "Pause continuous advancement",
        ("step", true) => "推进固定步数（payload.count）",
        ("step", false) => "Advance fixed steps (payload.count)",
        (_, true) => "未知动作",
        (_, false) => "Unknown action",
    }
}
#[cfg(target_arch = "wasm32")]
fn build_control_catalog_js_value() -> JsValue {
    let object = Object::new();
    let controls = Array::new();
    for action in WEB_TEST_API_CONTROL_ACTIONS {
        let entry = Object::new();
        let _ = JsReflect::set(
            &entry,
            &JsValue::from_str("action"),
            &JsValue::from_str(action),
        );
        let _ = JsReflect::set(
            &entry,
            &JsValue::from_str("description"),
            &JsValue::from_str(control_description(action, false)),
        );
        let _ = JsReflect::set(
            &entry,
            &JsValue::from_str("descriptionZh"),
            &JsValue::from_str(control_description(action, true)),
        );
        let _ = JsReflect::set(
            &entry,
            &JsValue::from_str("examplePayload"),
            &control_payload_example(action),
        );
        controls.push(&entry);
    }
    let _ = JsReflect::set(&object, &JsValue::from_str("controls"), &controls);
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("usage"),
        &JsValue::from_str("Use fillControlExample(action) then sendControl(action, payload)."),
    );
    JsValue::from(object)
}
#[cfg(target_arch = "wasm32")]
fn parse_control_example_action(payload: &JsValue) -> Option<String> {
    let action = parse_string_payload(payload)?;
    let action = action.trim().to_ascii_lowercase();
    WEB_TEST_API_CONTROL_ACTIONS
        .iter()
        .find(|candidate| **candidate == action)
        .map(|_| action)
}
#[cfg(target_arch = "wasm32")]
fn parse_control_action_label(control: &ViewerControl) -> String {
    match control {
        ViewerControl::Play => "play".to_string(),
        ViewerControl::Pause => "pause".to_string(),
        ViewerControl::Step { count } => format!("step(count={count})"),
        ViewerControl::Seek { tick } => format!("seek(tick={tick})"),
    }
}
#[cfg(target_arch = "wasm32")]
fn control_action_hint(action: &str, locale_zh: bool) -> String {
    match (action, locale_zh) {
        ("step", true) => "示例 payload: {\"count\": 5}".to_string(),
        ("step", false) => "Example payload: {\"count\": 5}".to_string(),
        (_, true) => "可用动作: play, pause, step".to_string(),
        (_, false) => "Valid actions: play, pause, step".to_string(),
    }
}
#[cfg(target_arch = "wasm32")]
fn update_last_control_feedback(feedback: WebTestApiControlFeedback) {
    WEB_TEST_API_STATE_SNAPSHOT.with(|slot| {
        slot.borrow_mut().last_control_feedback = Some(feedback);
    });
}

#[cfg(target_arch = "wasm32")]
fn mutate_last_control_feedback(
    feedback_id: u64,
    mutator: impl FnOnce(&mut WebTestApiControlFeedback),
) {
    WEB_TEST_API_STATE_SNAPSHOT.with(|slot| {
        let mut snapshot = slot.borrow_mut();
        let Some(current) = snapshot.last_control_feedback.as_mut() else {
            return;
        };
        if current.id == feedback_id {
            mutator(current);
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn parse_step_count(payload: &JsValue) -> Option<usize> {
    if payload.is_undefined() || payload.is_null() {
        return Some(1);
    }

    if let Some(number) = payload.as_f64() {
        if number.is_finite() && number >= 1.0 {
            return Some(number as usize);
        }
    }

    let count = JsReflect::get(payload, &JsValue::from_str("count")).ok()?;
    let number = count.as_f64()?;
    if number.is_finite() && number >= 1.0 {
        return Some(number as usize);
    }
    None
}

#[cfg(target_arch = "wasm32")]
fn parse_control_action(action: &str, payload: &JsValue) -> Option<ViewerControl> {
    match action.trim().to_ascii_lowercase().as_str() {
        "play" => Some(ViewerControl::Play),
        "pause" => Some(ViewerControl::Pause),
        "step" => parse_step_count(payload).map(|count| ViewerControl::Step { count }),
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
fn normalize_control_action(action: &str) -> String {
    action.trim().to_ascii_lowercase()
}

#[cfg(target_arch = "wasm32")]
fn parse_string_payload(payload: &JsValue) -> Option<String> {
    payload
        .as_string()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(target_arch = "wasm32")]
fn parse_run_steps_command(payload: &JsValue) -> Option<WebTestApiCommand> {
    if let Some(raw_steps) = parse_string_payload(payload) {
        let steps = parse_automation_steps(raw_steps.as_str());
        if steps.is_empty() {
            return None;
        }
        return Some(WebTestApiCommand::EnqueueSteps(steps));
    }

    parse_step_count(payload).map(|count| WebTestApiCommand::SendControl {
        control: ViewerControl::Step { count },
        feedback_id: 0,
    })
}

#[cfg(target_arch = "wasm32")]
fn build_control_feedback_js_value(feedback: &WebTestApiControlFeedback) -> JsValue {
    let object = Object::new();
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("id"),
        &JsValue::from_f64(feedback.id as f64),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("action"),
        &JsValue::from_str(feedback.action.as_str()),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("accepted"),
        &JsValue::from_bool(feedback.accepted),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("enqueued"),
        &JsValue::from_bool(feedback.enqueued),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("stage"),
        &JsValue::from_str(feedback.stage.as_str()),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("parsedControl"),
        &feedback
            .parsed_control
            .as_ref()
            .map(|value| JsValue::from_str(value))
            .unwrap_or(JsValue::NULL),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("reason"),
        &feedback
            .reason
            .as_ref()
            .map(|value| JsValue::from_str(value))
            .unwrap_or(JsValue::NULL),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("hint"),
        &feedback
            .hint
            .as_ref()
            .map(|value| JsValue::from_str(value))
            .unwrap_or(JsValue::NULL),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("effect"),
        &JsValue::from_str(feedback.effect.as_str()),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("deltaLogicalTime"),
        &JsValue::from_f64(feedback.delta_logical_time as f64),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("deltaEventSeq"),
        &JsValue::from_f64(feedback.delta_event_seq as f64),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("awaitingEffect"),
        &JsValue::from_bool(feedback.awaiting_effect),
    );
    JsValue::from(object)
}

#[cfg(target_arch = "wasm32")]
fn build_control_feedback(
    action: String,
    accepted: bool,
    parsed_control: Option<String>,
    reason: Option<String>,
    hint: Option<String>,
    effect: String,
    awaiting_effect: bool,
) -> WebTestApiControlFeedback {
    let (baseline_logical_time, baseline_event_seq) = latest_logical_time_event_seq();
    let stage = if !accepted {
        "blocked"
    } else if awaiting_effect {
        "received"
    } else {
        "applied"
    };
    WebTestApiControlFeedback {
        id: next_control_feedback_id(),
        action,
        accepted,
        enqueued: accepted && awaiting_effect,
        stage: stage.to_string(),
        parsed_control,
        reason,
        hint,
        effect,
        baseline_logical_time,
        baseline_event_seq,
        delta_logical_time: 0,
        delta_event_seq: 0,
        awaiting_effect,
        no_progress_frames: 0,
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn latest_web_test_api_control_feedback() -> Option<WebTestApiControlFeedbackSnapshot> {
    WEB_TEST_API_STATE_SNAPSHOT.with(|slot| {
        let snapshot = slot.borrow();
        snapshot
            .last_control_feedback
            .as_ref()
            .map(|feedback| WebTestApiControlFeedbackSnapshot {
                action: feedback.action.clone(),
                stage: feedback.stage.clone(),
                reason: feedback.reason.clone(),
                hint: feedback.hint.clone(),
                effect: feedback.effect.clone(),
            })
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn latest_web_test_api_control_feedback() -> Option<WebTestApiControlFeedbackSnapshot> {
    None
}

#[cfg(target_arch = "wasm32")]
fn build_state_js_value(snapshot: &WebTestApiStateSnapshot) -> JsValue {
    let object = Object::new();
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("connectionStatus"),
        &JsValue::from_str(snapshot.connection_status),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("logicalTime"),
        &JsValue::from_f64(snapshot.logical_time as f64),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("eventSeq"),
        &JsValue::from_f64(snapshot.event_seq as f64),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("tick"),
        &JsValue::from_f64(snapshot.logical_time as f64),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("selectedKind"),
        &snapshot
            .selected_kind
            .as_ref()
            .map(|value| JsValue::from_str(value))
            .unwrap_or(JsValue::NULL),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("selectedId"),
        &snapshot
            .selected_id
            .as_ref()
            .map(|value| JsValue::from_str(value))
            .unwrap_or(JsValue::NULL),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("errorCount"),
        &JsValue::from_f64(snapshot.error_count as f64),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("lastError"),
        &snapshot
            .last_error
            .as_ref()
            .map(|value| JsValue::from_str(value))
            .unwrap_or(JsValue::NULL),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("eventCount"),
        &JsValue::from_f64(snapshot.event_count as f64),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("traceCount"),
        &JsValue::from_f64(snapshot.trace_count as f64),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("cameraMode"),
        &JsValue::from_str(snapshot.camera_mode),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("cameraRadius"),
        &JsValue::from_f64(snapshot.camera_radius),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("cameraOrthoScale"),
        &JsValue::from_f64(snapshot.camera_ortho_scale),
    );
    let _ = JsReflect::set(
        &object,
        &JsValue::from_str("lastControlFeedback"),
        &snapshot
            .last_control_feedback
            .as_ref()
            .map(build_control_feedback_js_value)
            .unwrap_or(JsValue::NULL),
    );
    JsValue::from(object)
}

#[cfg(target_arch = "wasm32")]
pub(super) fn setup_web_test_api(world: &mut World) {
    let Some(window) = web_sys::window() else {
        return;
    };
    if !web_test_api_enabled(&window) {
        return;
    }

    let api = Object::new();

    let run_steps = Closure::wrap(Box::new(move |payload: JsValue| {
        let Some(command) = parse_run_steps_command(&payload) else {
            log_api_warning(
                "web test api: runSteps ignored (payload must be non-empty step string or count)",
            );
            return;
        };
        push_command(command);
    }) as Box<dyn FnMut(JsValue)>);
    let _ = JsReflect::set(
        &api,
        &JsValue::from_str("runSteps"),
        run_steps.as_ref().unchecked_ref(),
    );

    let set_mode = Closure::wrap(Box::new(move |payload: JsValue| {
        let Some(raw_mode) = parse_string_payload(&payload) else {
            log_api_warning("web test api: setMode ignored (mode must be non-empty string)");
            return;
        };
        let Some(mode) = parse_automation_mode(&raw_mode) else {
            log_api_warning("web test api: setMode ignored (invalid mode)");
            return;
        };
        push_command(WebTestApiCommand::EnqueueSteps(vec![
            crate::viewer_automation::ViewerAutomationStep::SetMode(mode),
        ]));
    }) as Box<dyn FnMut(JsValue)>);
    let _ = JsReflect::set(
        &api,
        &JsValue::from_str("setMode"),
        set_mode.as_ref().unchecked_ref(),
    );

    let focus = Closure::wrap(Box::new(move |payload: JsValue| {
        let Some(raw_target) = parse_string_payload(&payload) else {
            log_api_warning("web test api: focus ignored (target must be non-empty string)");
            return;
        };
        let Some(target) = parse_automation_target(&raw_target) else {
            log_api_warning("web test api: focus ignored (invalid target)");
            return;
        };
        push_command(WebTestApiCommand::EnqueueSteps(vec![
            crate::viewer_automation::ViewerAutomationStep::Focus(target),
        ]));
    }) as Box<dyn FnMut(JsValue)>);
    let _ = JsReflect::set(
        &api,
        &JsValue::from_str("focus"),
        focus.as_ref().unchecked_ref(),
    );

    let select = Closure::wrap(Box::new(move |payload: JsValue| {
        let Some(raw_target) = parse_string_payload(&payload) else {
            log_api_warning("web test api: select ignored (target must be non-empty string)");
            return;
        };
        let Some(target) = parse_automation_target(&raw_target) else {
            log_api_warning("web test api: select ignored (invalid target)");
            return;
        };
        push_command(WebTestApiCommand::EnqueueSteps(vec![
            crate::viewer_automation::ViewerAutomationStep::Select(target),
        ]));
    }) as Box<dyn FnMut(JsValue)>);
    let _ = JsReflect::set(
        &api,
        &JsValue::from_str("select"),
        select.as_ref().unchecked_ref(),
    );

    let describe_controls =
        Closure::wrap(
            Box::new(move || -> JsValue { build_control_catalog_js_value() })
                as Box<dyn FnMut() -> JsValue>,
        );
    let _ = JsReflect::set(
        &api,
        &JsValue::from_str("describeControls"),
        describe_controls.as_ref().unchecked_ref(),
    );

    let fill_control_example = Closure::wrap(Box::new(move |action: JsValue| -> JsValue {
        let Some(action) = parse_control_example_action(&action) else {
            log_api_warning("web test api: fillControlExample ignored (invalid action)");
            return JsValue::NULL;
        };
        let object = Object::new();
        let _ = JsReflect::set(
            &object,
            &JsValue::from_str("action"),
            &JsValue::from_str(action.as_str()),
        );
        let _ = JsReflect::set(
            &object,
            &JsValue::from_str("payload"),
            &control_payload_example(action.as_str()),
        );
        JsValue::from(object)
    }) as Box<dyn FnMut(JsValue) -> JsValue>);
    let _ = JsReflect::set(
        &api,
        &JsValue::from_str("fillControlExample"),
        fill_control_example.as_ref().unchecked_ref(),
    );

    let send_control = Closure::wrap(
        Box::new(move |action: JsValue, payload: JsValue| -> JsValue {
            let Some(raw_action) = parse_string_payload(&action) else {
                let feedback = build_control_feedback(
                    "<empty>".to_string(),
                    false,
                    None,
                    Some("action must be a non-empty string".to_string()),
                    Some(control_action_hint("unknown", false)),
                    "rejected before enqueue".to_string(),
                    false,
                );
                update_last_control_feedback(feedback.clone());
                log_api_warning(
                    "web test api: sendControl ignored (action must be non-empty string)",
                );
                return build_control_feedback_js_value(&feedback);
            };

            let action = normalize_control_action(raw_action.as_str());
            if !WEB_TEST_API_CONTROL_ACTIONS
                .iter()
                .any(|candidate| *candidate == action.as_str())
            {
                let feedback = build_control_feedback(
                    action.clone(),
                    false,
                    None,
                    Some(format!("unsupported action: {}", action)),
                    Some(control_action_hint("unknown", false)),
                    "rejected before enqueue".to_string(),
                    false,
                );
                update_last_control_feedback(feedback.clone());
                let warning =
                    format!("web test api: sendControl ignored (unsupported action: {action})");
                log_api_warning(warning.as_str());
                return build_control_feedback_js_value(&feedback);
            }

            let Some(control) = parse_control_action(action.as_str(), &payload) else {
                let reason = match action.as_str() {
                    "step" => "step requires numeric payload.count >= 1",
                    _ => "invalid payload for control action",
                };
                let feedback = build_control_feedback(
                    action.clone(),
                    false,
                    None,
                    Some(reason.to_string()),
                    Some(control_action_hint(action.as_str(), false)),
                    "rejected before enqueue".to_string(),
                    false,
                );
                update_last_control_feedback(feedback.clone());
                let warning = format!("web test api: sendControl ignored ({reason})");
                log_api_warning(warning.as_str());
                return build_control_feedback_js_value(&feedback);
            };
            let parsed_label = parse_control_action_label(&control);
            let feedback = build_control_feedback(
                action,
                true,
                Some(parsed_label),
                None,
                Some("queued, check getState().lastControlFeedback for world delta".to_string()),
                "queued control request".to_string(),
                true,
            );
            let feedback_id = feedback.id;
            update_last_control_feedback(feedback.clone());
            push_command(WebTestApiCommand::SendControl {
                control,
                feedback_id,
            });
            build_control_feedback_js_value(&feedback)
        }) as Box<dyn FnMut(JsValue, JsValue) -> JsValue>,
    );
    let _ = JsReflect::set(
        &api,
        &JsValue::from_str("sendControl"),
        send_control.as_ref().unchecked_ref(),
    );

    let get_state = Closure::wrap(Box::new(move || -> JsValue {
        WEB_TEST_API_STATE_SNAPSHOT.with(|slot| build_state_js_value(&slot.borrow()))
    }) as Box<dyn FnMut() -> JsValue>);
    let _ = JsReflect::set(
        &api,
        &JsValue::from_str("getState"),
        get_state.as_ref().unchecked_ref(),
    );

    let _ = JsReflect::set(&window, &JsValue::from_str(TEST_API_GLOBAL_NAME), &api);

    world.insert_non_send_resource(WebTestApiBindings {
        _api: api,
        _run_steps: run_steps,
        _set_mode: set_mode,
        _focus: focus,
        _select: select,
        _describe_controls: describe_controls,
        _fill_control_example: fill_control_example,
        _send_control: send_control,
        _get_state: get_state,
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn setup_web_test_api(_world: &mut World) {}

#[cfg(target_arch = "wasm32")]
pub(super) fn consume_web_test_api_commands(
    mut automation_state: ResMut<ViewerAutomationState>,
    _state: Option<Res<ViewerState>>,
    client: Option<Res<ViewerClient>>,
    control_profile: Option<Res<ViewerControlProfileState>>,
) {
    let mut commands = Vec::new();
    WEB_TEST_API_COMMAND_QUEUE.with(|queue| {
        let mut queue = queue.borrow_mut();
        while let Some(command) = queue.pop_front() {
            commands.push(command);
        }
    });

    for command in commands {
        match command {
            WebTestApiCommand::EnqueueSteps(steps) => {
                enqueue_runtime_steps(&mut automation_state, steps);
            }
            WebTestApiCommand::SendControl {
                control,
                feedback_id,
            } => {
                let Some(client) = client.as_deref() else {
                    mutate_last_control_feedback(feedback_id, |feedback| {
                        feedback.accepted = false;
                        feedback.enqueued = false;
                        feedback.stage = "blocked".to_string();
                        feedback.reason = Some("viewer client is not available".to_string());
                        feedback.hint = Some("reconnect then retry sendControl".to_string());
                        feedback.effect = "dropped before dispatch".to_string();
                        feedback.awaiting_effect = false;
                    });
                    continue;
                };
                let sent = dispatch_viewer_control(client, control_profile.as_deref(), control);
                mutate_last_control_feedback(feedback_id, |feedback| {
                    if sent {
                        feedback.stage = "executing".to_string();
                        feedback.enqueued = true;
                        feedback.hint =
                            Some("dispatch accepted, waiting for world delta".to_string());
                    } else {
                        feedback.accepted = false;
                        feedback.enqueued = false;
                        feedback.stage = "blocked".to_string();
                        feedback.reason = Some("viewer client channel send failed".to_string());
                        feedback.hint = Some("retry control after reconnect".to_string());
                        feedback.effect = "dropped before dispatch".to_string();
                        feedback.awaiting_effect = false;
                    }
                });
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn consume_web_test_api_commands(
    _automation_state: ResMut<ViewerAutomationState>,
    _state: Option<Res<ViewerState>>,
    _client: Option<Res<ViewerClient>>,
    _control_profile: Option<Res<ViewerControlProfileState>>,
) {
}

#[cfg(target_arch = "wasm32")]
pub(super) fn publish_web_test_api_state(
    state: Res<ViewerState>,
    selection: Res<ViewerSelection>,
    camera_mode: Res<ViewerCameraMode>,
    cameras: Query<(&OrbitCamera, &Projection), With<Viewer3dCamera>>,
    client: Option<Res<ViewerClient>>,
    control_profile: Option<Res<ViewerControlProfileState>>,
) {
    WEB_TEST_API_STATE_SNAPSHOT.with(|slot| {
        let mut snapshot = slot.borrow_mut();
        snapshot.connection_status = match &state.status {
            ConnectionStatus::Connecting => "connecting",
            ConnectionStatus::Connected => "connected",
            ConnectionStatus::Error(_) => "error",
        };
        let snapshot_tick = state
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.time)
            .unwrap_or(0);
        let metrics_tick = state
            .metrics
            .as_ref()
            .map(|metrics| metrics.total_ticks)
            .unwrap_or(0);
        snapshot.logical_time = snapshot_tick.max(metrics_tick);
        snapshot.event_seq = state
            .events
            .iter()
            .map(|event| event.id)
            .max()
            .unwrap_or(snapshot.event_seq);
        snapshot.event_count = state.events.len();
        snapshot.trace_count = state.decision_traces.len();
        snapshot.selected_kind = selection
            .current
            .as_ref()
            .map(|info| match info.kind {
                SelectionKind::Agent => "agent",
                SelectionKind::Location => "location",
                SelectionKind::Fragment => "fragment",
                SelectionKind::Asset => "asset",
                SelectionKind::PowerPlant => "power_plant",
                SelectionKind::PowerStorage => "power_storage",
                SelectionKind::Chunk => "chunk",
            })
            .map(str::to_string);
        snapshot.selected_id = selection.current.as_ref().map(|info| info.id.clone());

        match &state.status {
            ConnectionStatus::Error(message) => {
                if snapshot.last_error.as_deref() != Some(message.as_str()) {
                    snapshot.error_count = snapshot.error_count.saturating_add(1);
                }
                snapshot.last_error = Some(message.clone());
            }
            _ => {
                snapshot.last_error = None;
            }
        }

        snapshot.camera_mode = match *camera_mode {
            ViewerCameraMode::TwoD => "2d",
            ViewerCameraMode::ThreeD => "3d",
        };

        if let Ok((orbit, projection)) = cameras.single() {
            snapshot.camera_radius = orbit.radius as f64;
            snapshot.camera_ortho_scale = match projection {
                Projection::Orthographic(ortho) => ortho.scale as f64,
                _ => 0.0,
            };
        } else {
            snapshot.camera_radius = 0.0;
            snapshot.camera_ortho_scale = 0.0;
        }

        let connection_ready = matches!(state.status, ConnectionStatus::Connected);
        let latest_logical_time = snapshot.logical_time;
        let latest_event_seq = snapshot.event_seq;
        if let Some(feedback) = snapshot.last_control_feedback.as_mut() {
            if feedback.awaiting_effect {
                let delta_logical_time =
                    latest_logical_time.saturating_sub(feedback.baseline_logical_time);
                let delta_event_seq = latest_event_seq.saturating_sub(feedback.baseline_event_seq);
                feedback.delta_logical_time = delta_logical_time;
                feedback.delta_event_seq = delta_event_seq;
                if delta_logical_time > 0 || delta_event_seq > 0 {
                    feedback.stage = "applied".to_string();
                    feedback.no_progress_frames = 0;
                    feedback.effect =
                        format!("world advanced: logicalTime +{delta_logical_time}, eventSeq +{delta_event_seq}");
                    feedback.hint = Some("input was accepted and world state advanced".to_string());
                    feedback.awaiting_effect = false;
                } else if connection_ready {
                    feedback.no_progress_frames = feedback.no_progress_frames.saturating_add(1);
                    if feedback.no_progress_frames >= CONTROL_STALL_FRAME_THRESHOLD {
                        if feedback.action == "step" {
                            let recovered = client.as_ref().is_some_and(|viewer_client| {
                                dispatch_viewer_control(
                                    viewer_client,
                                    control_profile.as_deref(),
                                    ViewerControl::Play,
                                )
                            });
                            feedback.reason = Some(format!(
                                "Cause: step accepted but no world delta within stall window ({} frames)",
                                CONTROL_STALL_FRAME_THRESHOLD
                            ));
                            if recovered {
                                feedback.stage = "executing".to_string();
                                feedback.hint = Some(
                                    "Next: auto recovery dispatched play; if still stalled, retry step"
                                        .to_string(),
                                );
                                feedback.effect = "step stalled, auto recovery play dispatched"
                                    .to_string();
                                feedback.baseline_logical_time = latest_logical_time;
                                feedback.baseline_event_seq = latest_event_seq;
                                feedback.no_progress_frames = 0;
                            } else {
                                feedback.stage = "blocked".to_string();
                                feedback.hint = Some("Next: use play, then retry step".to_string());
                                feedback.effect = "accepted without observed progress".to_string();
                                feedback.awaiting_effect = false;
                            }
                        } else {
                            feedback.stage = "blocked".to_string();
                            feedback.reason =
                                Some("Cause: no world delta observed while connected".to_string());
                            feedback.hint = Some(control_action_hint(feedback.action.as_str(), false));
                            feedback.effect = "accepted without observed progress".to_string();
                            feedback.awaiting_effect = false;
                        }
                    } else {
                        feedback.stage = "executing".to_string();
                        feedback.effect = "queued, waiting for next world delta".to_string();
                    }
                } else {
                    feedback.stage = "received".to_string();
                    feedback.effect = "queued, waiting for world connection".to_string();
                }
            }
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn publish_web_test_api_state(
    _state: Res<ViewerState>,
    _selection: Res<ViewerSelection>,
    _camera_mode: Res<ViewerCameraMode>,
    _cameras: Query<(&OrbitCamera, &Projection), With<Viewer3dCamera>>,
    _client: Option<Res<ViewerClient>>,
) {
}
