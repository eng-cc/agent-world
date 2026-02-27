use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::viewer_automation::{
    enqueue_runtime_steps, parse_automation_mode, parse_automation_steps, parse_automation_target,
};
#[cfg(target_arch = "wasm32")]
use crate::{ConnectionStatus, SelectionKind};
use crate::{OrbitCamera, Viewer3dCamera, ViewerCameraMode};
use crate::{ViewerAutomationState, ViewerClient, ViewerSelection, ViewerState};
#[cfg(target_arch = "wasm32")]
use agent_world::viewer::{ViewerControl, ViewerRequest};

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::collections::VecDeque;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys::{Object, Reflect as JsReflect};
#[cfg(target_arch = "wasm32")]
use web_sys::UrlSearchParams;

#[cfg(target_arch = "wasm32")]
const TEST_API_QUERY_KEY: &str = "test_api";
#[cfg(target_arch = "wasm32")]
const TEST_API_GLOBAL_NAME: &str = "__AW_TEST__";

#[cfg(target_arch = "wasm32")]
enum WebTestApiCommand {
    EnqueueSteps(Vec<crate::viewer_automation::ViewerAutomationStep>),
    SendControl(ViewerControl),
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
        }
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WEB_TEST_API_COMMAND_QUEUE: RefCell<VecDeque<WebTestApiCommand>> = RefCell::new(VecDeque::new());
    static WEB_TEST_API_STATE_SNAPSHOT: RefCell<WebTestApiStateSnapshot> = RefCell::new(WebTestApiStateSnapshot::default());
}

#[cfg(target_arch = "wasm32")]
pub(super) struct WebTestApiBindings {
    _api: Object,
    _run_steps: Closure<dyn FnMut(JsValue)>,
    _set_mode: Closure<dyn FnMut(JsValue)>,
    _focus: Closure<dyn FnMut(JsValue)>,
    _select: Closure<dyn FnMut(JsValue)>,
    _send_control: Closure<dyn FnMut(JsValue, JsValue)>,
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
fn parse_seek_tick(payload: &JsValue) -> Option<u64> {
    if let Some(number) = payload.as_f64() {
        if number.is_finite() && number >= 0.0 {
            return Some(number as u64);
        }
    }
    let tick = JsReflect::get(payload, &JsValue::from_str("tick")).ok()?;
    let number = tick.as_f64()?;
    if number.is_finite() && number >= 0.0 {
        return Some(number as u64);
    }
    None
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
        "seek" => parse_seek_tick(payload).map(|tick| ViewerControl::Seek { tick }),
        _ => None,
    }
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

    parse_step_count(payload)
        .map(|count| WebTestApiCommand::SendControl(ViewerControl::Step { count }))
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

    let send_control = Closure::wrap(Box::new(move |action: JsValue, payload: JsValue| {
        let Some(action) = parse_string_payload(&action) else {
            log_api_warning("web test api: sendControl ignored (action must be non-empty string)");
            return;
        };
        let Some(control) = parse_control_action(action.as_str(), &payload) else {
            log_api_warning("web test api: sendControl ignored (invalid action or payload)");
            return;
        };
        push_command(WebTestApiCommand::SendControl(control));
    }) as Box<dyn FnMut(JsValue, JsValue)>);
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
        _send_control: send_control,
        _get_state: get_state,
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn setup_web_test_api(_world: &mut World) {}

#[cfg(target_arch = "wasm32")]
pub(super) fn consume_web_test_api_commands(
    mut automation_state: ResMut<ViewerAutomationState>,
    client: Option<Res<ViewerClient>>,
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
            WebTestApiCommand::SendControl(control) => {
                if let Some(client) = client.as_deref() {
                    let _ = client.tx.send(ViewerRequest::Control { mode: control });
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn consume_web_test_api_commands(
    _automation_state: ResMut<ViewerAutomationState>,
    _client: Option<Res<ViewerClient>>,
) {
}

#[cfg(target_arch = "wasm32")]
pub(super) fn publish_web_test_api_state(
    state: Res<ViewerState>,
    selection: Res<ViewerSelection>,
    camera_mode: Res<ViewerCameraMode>,
    cameras: Query<(&OrbitCamera, &Projection), With<Viewer3dCamera>>,
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
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn publish_web_test_api_state(
    _state: Res<ViewerState>,
    _selection: Res<ViewerSelection>,
    _camera_mode: Res<ViewerCameraMode>,
    _cameras: Query<(&OrbitCamera, &Projection), With<Viewer3dCamera>>,
) {
}
