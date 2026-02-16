use bevy::prelude::*;
use bevy::window::RequestRedraw;
use bevy_egui::input::EguiInputEvent;
use bevy_egui::{egui, EguiContext, EguiOutput, PrimaryEguiContext};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CompositionEvent, HtmlInputElement, InputEvent, KeyboardEvent};

const EGUI_IME_BRIDGE_INPUT_ID: &str = "egui_text_agent";
const EGUI_IME_BRIDGE_STYLE: &str = "background-color:transparent;border:none;outline:none;width:1px;height:1px;caret-color:transparent;position:absolute;top:0;left:0;";

#[derive(Clone, Copy, Default)]
struct BridgeInputState {
    composing: bool,
    suppress_next_input: bool,
}

struct WasmEguiInputBridgeClosures {
    _input: Closure<dyn FnMut(InputEvent)>,
    _composition_start: Closure<dyn FnMut(CompositionEvent)>,
    _composition_update: Closure<dyn FnMut(CompositionEvent)>,
    _composition_end: Closure<dyn FnMut(CompositionEvent)>,
    _keydown: Closure<dyn FnMut(KeyboardEvent)>,
    _keyup: Closure<dyn FnMut(KeyboardEvent)>,
}

pub(super) struct WasmEguiInputBridgeState {
    input: HtmlInputElement,
    rx: Receiver<egui::Event>,
    focused: bool,
    _closures: WasmEguiInputBridgeClosures,
}

pub(super) fn setup_wasm_egui_input_bridge(world: &mut World) {
    let Some(window) = web_sys::window() else {
        warn!("wasm ime bridge: missing window");
        return;
    };
    let Some(document) = window.document() else {
        warn!("wasm ime bridge: missing document");
        return;
    };
    let Some(body) = document.body() else {
        warn!("wasm ime bridge: missing document body");
        return;
    };

    let (input, created_here) = match document
        .get_element_by_id(EGUI_IME_BRIDGE_INPUT_ID)
        .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
    {
        Some(input) => (input, false),
        None => {
            let Ok(element) = document.create_element("input") else {
                warn!("wasm ime bridge: failed to create input element");
                return;
            };
            let Ok(input) = element.dyn_into::<HtmlInputElement>() else {
                warn!("wasm ime bridge: failed to create input element");
                return;
            };
            (input, true)
        }
    };

    if created_here {
        input.set_id(EGUI_IME_BRIDGE_INPUT_ID);
        input.set_type("text");
        input.set_tab_index(-1);
        input.set_hidden(true);
        input.set_size(1);
        let _ = input.set_attribute("style", EGUI_IME_BRIDGE_STYLE);
        let _ = input.set_attribute("autocapitalize", "off");
        let _ = input.set_attribute("autocomplete", "off");
        let _ = input.set_attribute("autocorrect", "off");
        let _ = input.set_attribute("spellcheck", "false");

        if body.append_child(&input).is_err() {
            warn!("wasm ime bridge: failed to append input element");
            return;
        }
    }

    let (tx, rx) = mpsc::channel::<egui::Event>();
    let state = Rc::new(RefCell::new(BridgeInputState::default()));

    let input_for_input = input.clone();
    let tx_for_input = tx.clone();
    let state_for_input = state.clone();
    let input_closure = Closure::wrap(Box::new(move |event: InputEvent| {
        let mut state = state_for_input.borrow_mut();
        if state.composing || event.is_composing() {
            return;
        }

        let text = input_for_input.value();
        if text.is_empty() {
            return;
        }
        input_for_input.set_value("");

        if state.suppress_next_input {
            state.suppress_next_input = false;
            return;
        }

        let _ = tx_for_input.send(egui::Event::Text(text));
    }) as Box<dyn FnMut(_)>);
    if input
        .add_event_listener_with_callback("input", input_closure.as_ref().unchecked_ref())
        .is_err()
    {
        warn!("wasm ime bridge: failed to register input listener");
        return;
    }

    let input_for_start = input.clone();
    let tx_for_start = tx.clone();
    let state_for_start = state.clone();
    let composition_start_closure = Closure::wrap(Box::new(move |_event: CompositionEvent| {
        let mut state = state_for_start.borrow_mut();
        state.composing = true;
        state.suppress_next_input = false;
        input_for_start.set_value("");
        let _ = tx_for_start.send(egui::Event::Ime(egui::ImeEvent::Enabled));
    }) as Box<dyn FnMut(_)>);
    if input
        .add_event_listener_with_callback(
            "compositionstart",
            composition_start_closure.as_ref().unchecked_ref(),
        )
        .is_err()
    {
        warn!("wasm ime bridge: failed to register compositionstart listener");
        return;
    }

    let tx_for_update = tx.clone();
    let composition_update_closure = Closure::wrap(Box::new(move |event: CompositionEvent| {
        if let Some(text) = event.data() {
            let _ = tx_for_update.send(egui::Event::Ime(egui::ImeEvent::Preedit(text)));
        }
    }) as Box<dyn FnMut(_)>);
    if input
        .add_event_listener_with_callback(
            "compositionupdate",
            composition_update_closure.as_ref().unchecked_ref(),
        )
        .is_err()
    {
        warn!("wasm ime bridge: failed to register compositionupdate listener");
        return;
    }

    let tx_for_end = tx.clone();
    let state_for_end = state.clone();
    let composition_end_closure = Closure::wrap(Box::new(move |event: CompositionEvent| {
        let mut state = state_for_end.borrow_mut();
        state.composing = false;

        if let Some(text) = event.data() {
            if !text.is_empty() {
                state.suppress_next_input = true;
                let _ = tx_for_end.send(egui::Event::Ime(egui::ImeEvent::Commit(text)));
            }
        }

        let _ = tx_for_end.send(egui::Event::Ime(egui::ImeEvent::Disabled));
    }) as Box<dyn FnMut(_)>);
    if input
        .add_event_listener_with_callback(
            "compositionend",
            composition_end_closure.as_ref().unchecked_ref(),
        )
        .is_err()
    {
        warn!("wasm ime bridge: failed to register compositionend listener");
        return;
    }

    let tx_for_keydown = tx.clone();
    let keydown_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        if event.is_composing() {
            return;
        }
        let Some(key) = map_web_key(&event.key()) else {
            return;
        };
        let _ = tx_for_keydown.send(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: event.repeat(),
            modifiers: modifiers_from_web_event(&event),
        });
    }) as Box<dyn FnMut(_)>);
    if input
        .add_event_listener_with_callback("keydown", keydown_closure.as_ref().unchecked_ref())
        .is_err()
    {
        warn!("wasm ime bridge: failed to register keydown listener");
        return;
    }

    let tx_for_keyup = tx.clone();
    let keyup_closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        if event.is_composing() {
            return;
        }
        let Some(key) = map_web_key(&event.key()) else {
            return;
        };
        let _ = tx_for_keyup.send(egui::Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: modifiers_from_web_event(&event),
        });
    }) as Box<dyn FnMut(_)>);
    if input
        .add_event_listener_with_callback("keyup", keyup_closure.as_ref().unchecked_ref())
        .is_err()
    {
        warn!("wasm ime bridge: failed to register keyup listener");
        return;
    }

    world.insert_non_send_resource(WasmEguiInputBridgeState {
        input,
        rx,
        focused: false,
        _closures: WasmEguiInputBridgeClosures {
            _input: input_closure,
            _composition_start: composition_start_closure,
            _composition_update: composition_update_closure,
            _composition_end: composition_end_closure,
            _keydown: keydown_closure,
            _keyup: keyup_closure,
        },
    });
}

pub(super) fn sync_wasm_egui_input_bridge_focus(
    bridge: Option<NonSendMut<WasmEguiInputBridgeState>>,
    chat_focus_signal: Option<Res<crate::ChatInputFocusSignal>>,
    mut context_query: Query<(&EguiOutput, &mut EguiContext), With<PrimaryEguiContext>>,
) {
    let Some(mut bridge) = bridge else {
        return;
    };
    let Ok((output, mut context)) = context_query.single_mut() else {
        return;
    };

    let chat_input_focused = context
        .get_mut()
        .memory(|memory| memory.has_focus(egui::Id::new(crate::EGUI_CHAT_INPUT_WIDGET_ID)));
    let chat_focus_signal = chat_focus_signal
        .as_deref()
        .map(|signal| signal.wants_ime_focus)
        .unwrap_or(false);
    let wants_keyboard_input = context.get_mut().wants_keyboard_input();
    let editing_text = chat_focus_signal
        || chat_input_focused
        || wants_keyboard_input
        || output.platform_output.ime.is_some()
        || output.platform_output.mutable_text_under_cursor;
    let dom_focused = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element())
        .map(|element| element.id() == EGUI_IME_BRIDGE_INPUT_ID)
        .unwrap_or(false);

    if editing_text {
        if bridge.focused && dom_focused {
            return;
        }
        bridge.input.set_hidden(false);
        if !dom_focused {
            let _ = bridge.input.focus();
        }
        bridge.focused = true;
        return;
    }

    if !bridge.focused && !dom_focused {
        return;
    }

    bridge.input.set_value("");
    if dom_focused {
        let _ = bridge.input.blur();
    }
    bridge.input.set_hidden(true);
    bridge.focused = false;
}

pub(super) fn pump_wasm_egui_input_bridge_events(
    bridge: Option<NonSendMut<WasmEguiInputBridgeState>>,
    contexts: Query<Entity, With<PrimaryEguiContext>>,
    mut writer: MessageWriter<EguiInputEvent>,
    mut redraw_writer: MessageWriter<RequestRedraw>,
) {
    let Some(bridge) = bridge else {
        return;
    };
    let Ok(context) = contexts.single() else {
        return;
    };

    let mut emitted = false;
    while let Ok(event) = bridge.rx.try_recv() {
        writer.write(EguiInputEvent { context, event });
        emitted = true;
    }

    if emitted {
        redraw_writer.write(RequestRedraw);
    }
}

fn map_web_key(raw: &str) -> Option<egui::Key> {
    Some(match raw {
        "ArrowDown" => egui::Key::ArrowDown,
        "ArrowLeft" => egui::Key::ArrowLeft,
        "ArrowRight" => egui::Key::ArrowRight,
        "ArrowUp" => egui::Key::ArrowUp,
        "Backspace" => egui::Key::Backspace,
        "Delete" => egui::Key::Delete,
        "End" => egui::Key::End,
        "Enter" => egui::Key::Enter,
        "Escape" => egui::Key::Escape,
        "Home" => egui::Key::Home,
        "Insert" => egui::Key::Insert,
        "PageDown" => egui::Key::PageDown,
        "PageUp" => egui::Key::PageUp,
        "Tab" => egui::Key::Tab,
        _ => return None,
    })
}

fn modifiers_from_web_event(event: &KeyboardEvent) -> egui::Modifiers {
    let alt = event.alt_key();
    let ctrl = event.ctrl_key();
    let shift = event.shift_key();
    let mac_cmd = event.meta_key();
    let command = ctrl || mac_cmd;
    egui::Modifiers {
        alt,
        ctrl,
        shift,
        mac_cmd,
        command,
    }
}
