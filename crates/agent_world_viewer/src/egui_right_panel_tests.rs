use super::egui_observe_section_card::{section_tone, ObserveSectionTone};
use super::*;
use crate::right_panel_module_visibility::RightPanelModuleVisibilityState;
use crate::ViewerControl;
use agent_world::simulator::{RejectReason, WorldEvent, WorldEventKind};
use egui_kittest::{kittest::Queryable as _, Harness};
use egui_wgpu::wgpu;
use std::iter::once;
use std::mem::size_of;
use std::sync::mpsc::channel;
use std::time::Duration;

const SNAPSHOT_OUTPUT_DIR: &str = "tests/snapshots";
const SNAPSHOT_WAIT_TIMEOUT: Duration = Duration::from_secs(10);

struct SnapshotRenderer {
    render_state: egui_wgpu::RenderState,
}

impl SnapshotRenderer {
    fn try_new() -> Result<Self, String> {
        let setup = egui_wgpu::WgpuSetup::CreateNew(egui_wgpu::WgpuSetupCreateNew::default());
        let instance = pollster::block_on(setup.new_instance());
        let render_state = pollster::block_on(egui_wgpu::RenderState::create(
            &egui_wgpu::WgpuConfiguration {
                wgpu_setup: setup,
                ..Default::default()
            },
            &instance,
            None,
            egui_wgpu::RendererOptions::PREDICTABLE,
        ))
        .map_err(|err| format!("failed to create wgpu render state for snapshots: {err}"))?;

        Ok(Self { render_state })
    }
}

fn snapshot_renderer_or_skip() -> Option<SnapshotRenderer> {
    match SnapshotRenderer::try_new() {
        Ok(renderer) => Some(renderer),
        Err(err) => {
            eprintln!("skip egui snapshot tests because wgpu is unavailable: {err}");
            None
        }
    }
}

impl egui_kittest::TestRenderer for SnapshotRenderer {
    fn handle_delta(&mut self, delta: &egui::TexturesDelta) {
        let mut renderer = self.render_state.renderer.write();
        for (texture_id, image_delta) in &delta.set {
            renderer.update_texture(
                &self.render_state.device,
                &self.render_state.queue,
                *texture_id,
                image_delta,
            );
        }
        for texture_id in &delta.free {
            renderer.free_texture(texture_id);
        }
    }

    fn render(
        &mut self,
        ctx: &egui::Context,
        output: &egui::FullOutput,
    ) -> Result<image::RgbaImage, String> {
        let mut renderer = self.render_state.renderer.write();
        let mut encoder =
            self.render_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("EguiKittestSnapshotEncoder"),
                });

        let size = ctx.content_rect().size() * ctx.pixels_per_point();
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            pixels_per_point: ctx.pixels_per_point(),
            size_in_pixels: [size.x.round() as u32, size.y.round() as u32],
        };
        let clipped_primitives = ctx.tessellate(output.shapes.clone(), ctx.pixels_per_point());

        let user_cmd_bufs = renderer.update_buffers(
            &self.render_state.device,
            &self.render_state.queue,
            &mut encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        let texture = self
            .render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("EguiKittestSnapshotTexture"),
                size: wgpu::Extent3d {
                    width: screen_descriptor.size_in_pixels[0],
                    height: screen_descriptor.size_in_pixels[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.render_state.target_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("EguiKittestSnapshotPass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    ..Default::default()
                })
                .forget_lifetime();
            renderer.render(&mut pass, &clipped_primitives, &screen_descriptor);
        }

        self.render_state
            .queue
            .submit(user_cmd_bufs.into_iter().chain(once(encoder.finish())));
        self.render_state
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(SNAPSHOT_WAIT_TIMEOUT),
            })
            .map_err(|err| format!("poll error while rendering snapshot: {err}"))?;

        Ok(texture_to_image(
            &self.render_state.device,
            &self.render_state.queue,
            &texture,
        ))
    }
}

fn texture_to_image(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> image::RgbaImage {
    let dims = BufferDimensions::new(texture.width() as usize, texture.height() as usize);
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("EguiKittestSnapshotReadback"),
        size: (dims.padded_bytes_per_row * dims.height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("EguiKittestSnapshotCopyEncoder"),
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(dims.padded_bytes_per_row as u32),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: texture.width(),
            height: texture.height(),
            depth_or_array_layers: 1,
        },
    );

    let submit_index = queue.submit(once(encoder.finish()));
    let slice = output_buffer.slice(..);
    let (tx, rx) = channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submit_index),
            timeout: Some(SNAPSHOT_WAIT_TIMEOUT),
        })
        .expect("failed to poll wgpu device for snapshot readback");
    rx.recv()
        .expect("snapshot channel closed")
        .expect("failed to map snapshot buffer");

    let mapped = output_buffer.slice(..).get_mapped_range();
    let bytes = mapped
        .chunks_exact(dims.padded_bytes_per_row)
        .flat_map(|row| row.iter().take(dims.unpadded_bytes_per_row))
        .copied()
        .collect::<Vec<_>>();
    drop(mapped);
    output_buffer.unmap();

    image::RgbaImage::from_raw(texture.width(), texture.height(), bytes)
        .expect("failed to build image from snapshot bytes")
}

struct BufferDimensions {
    height: usize,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let bytes_per_pixel = size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        Self {
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row: unpadded_bytes_per_row + padding,
        }
    }
}

fn snapshot_options() -> egui_kittest::SnapshotOptions {
    egui_kittest::SnapshotOptions::new()
        .threshold(0.6)
        .failed_pixel_count_threshold(
            egui_kittest::OsThreshold::new(0)
                .macos(24)
                .linux(24)
                .windows(24),
        )
        .output_path(SNAPSHOT_OUTPUT_DIR)
}

fn sample_rejected_event(id: u64, time: u64) -> WorldEvent {
    WorldEvent {
        id,
        time,
        kind: WorldEventKind::ActionRejected {
            reason: RejectReason::AgentNotFound {
                agent_id: format!("agent-{id}"),
            },
        },
    }
}

fn sample_agent_moved_event(id: u64, time: u64) -> WorldEvent {
    WorldEvent {
        id,
        time,
        kind: WorldEventKind::AgentMoved {
            agent_id: format!("agent-{id}"),
            from: "from-loc".to_string(),
            to: "to-loc".to_string(),
            distance_cm: 12,
            electricity_cost: 3,
        },
    }
}

fn sample_viewer_state(
    status: crate::ConnectionStatus,
    events: Vec<WorldEvent>,
) -> crate::ViewerState {
    crate::ViewerState {
        status,
        snapshot: None,
        events,
        decision_traces: Vec::new(),
        metrics: None,
    }
}

#[test]
fn adaptive_panel_width_clamps_to_bounds() {
    assert_eq!(adaptive_panel_default_width(200.0), MAIN_PANEL_MIN_WIDTH);
    assert_eq!(adaptive_panel_default_width(1200.0), 264.0);
    assert_eq!(adaptive_panel_default_width(1500.0), 330.0);
    assert_eq!(adaptive_panel_default_width(4000.0), 880.0);
}

#[test]
fn adaptive_panel_max_width_scales_with_available_width() {
    assert_eq!(adaptive_panel_max_width(200.0), MAIN_PANEL_MIN_WIDTH);
    assert_eq!(adaptive_panel_max_width(1200.0), 720.0);
    assert_eq!(adaptive_panel_max_width(2000.0), 1200.0);
}

#[test]
fn adaptive_chat_panel_width_clamps_to_bounds() {
    assert_eq!(
        adaptive_chat_panel_default_width(200.0),
        CHAT_PANEL_MIN_WIDTH
    );
    assert_eq!(adaptive_chat_panel_default_width(1200.0), 300.0);
    assert_eq!(adaptive_chat_panel_default_width(1800.0), 450.0);
    assert_eq!(adaptive_chat_panel_default_width(4000.0), 1000.0);
}

#[test]
fn adaptive_chat_panel_max_width_scales_with_available_width() {
    assert_eq!(adaptive_chat_panel_max_width(200.0), CHAT_PANEL_MIN_WIDTH);
    assert_eq!(adaptive_chat_panel_max_width(1200.0), 780.0);
    assert_eq!(adaptive_chat_panel_max_width(2000.0), 1300.0);
}

#[test]
fn compact_chat_layout_switches_below_breakpoint() {
    assert!(is_compact_chat_layout(390.0));
    assert!(is_compact_chat_layout(
        CHAT_SIDE_PANEL_COMPACT_BREAKPOINT - 1.0
    ));
    assert!(!is_compact_chat_layout(
        CHAT_SIDE_PANEL_COMPACT_BREAKPOINT + 1.0
    ));
}

#[test]
fn adaptive_main_panel_min_width_uses_compact_floor_on_narrow_width() {
    assert_eq!(
        adaptive_main_panel_min_width(390.0),
        MAIN_PANEL_COMPACT_MIN_WIDTH
    );
    assert_eq!(adaptive_main_panel_min_width(1280.0), MAIN_PANEL_MIN_WIDTH);
}

#[test]
fn adaptive_chat_panel_max_width_for_side_layout_respects_viewport_budget() {
    assert_eq!(adaptive_chat_panel_max_width_for_side_layout(500.0), 20.0);
    assert_eq!(adaptive_chat_panel_max_width_for_side_layout(760.0), 280.0);
    assert_eq!(adaptive_chat_panel_max_width_for_side_layout(1200.0), 720.0);
}

#[test]
fn adaptive_main_panel_max_width_for_layout_respects_interaction_budget() {
    assert_eq!(
        adaptive_main_panel_max_width_for_layout(1365.0, 360.0),
        765.0
    );
    assert_eq!(
        adaptive_main_panel_max_width_for_layout(390.0, 0.0),
        MAIN_PANEL_COMPACT_MIN_WIDTH
    );
}

#[test]
fn show_chat_panel_requires_expanded_top_and_visibility_enabled() {
    let expanded_layout = RightPanelLayoutState {
        top_panel_collapsed: false,
        panel_hidden: false,
    };
    assert!(should_show_chat_panel(&expanded_layout, true));
    assert!(!should_show_chat_panel(&expanded_layout, false));

    let collapsed_layout = RightPanelLayoutState {
        top_panel_collapsed: true,
        panel_hidden: false,
    };
    assert!(!should_show_chat_panel(&collapsed_layout, true));

    let hidden_layout = RightPanelLayoutState {
        top_panel_collapsed: false,
        panel_hidden: true,
    };
    assert!(!should_show_chat_panel(&hidden_layout, true));
}

#[test]
fn total_right_panel_width_adds_main_and_chat_width() {
    assert_eq!(total_right_panel_width(320.0, 360.0), 680.0);
    assert_eq!(total_right_panel_width(320.0, 0.0), 320.0);
    assert_eq!(total_right_panel_width(-10.0, 100.0), 100.0);
}

#[test]
fn env_toggle_enabled_parses_truthy_values() {
    assert!(env_toggle_enabled(Some("1")));
    assert!(env_toggle_enabled(Some(" true ")));
    assert!(env_toggle_enabled(Some("YES")));
    assert!(env_toggle_enabled(Some("on")));
}

#[test]
fn env_toggle_enabled_rejects_falsy_values() {
    assert!(!env_toggle_enabled(None));
    assert!(!env_toggle_enabled(Some("0")));
    assert!(!env_toggle_enabled(Some("false")));
    assert!(!env_toggle_enabled(Some("off")));
    assert!(!env_toggle_enabled(Some("")));
}

#[test]
fn section_tone_maps_titles_for_zh_and_en() {
    assert_eq!(section_tone("World Summary"), ObserveSectionTone::World);
    assert_eq!(section_tone("Agent Activity"), ObserveSectionTone::Activity);
    assert_eq!(
        section_tone("Industrial Ops"),
        ObserveSectionTone::Industrial
    );
    assert_eq!(
        section_tone("Economy Dashboard"),
        ObserveSectionTone::Economy
    );
    assert_eq!(section_tone("Ops Navigator"), ObserveSectionTone::Ops);
    assert_eq!(section_tone("选中详情"), ObserveSectionTone::Details);
    assert_eq!(section_tone("事件"), ObserveSectionTone::Events);
    assert_eq!(section_tone("Random Title"), ObserveSectionTone::Default);
}

#[test]
fn connection_signal_matches_status() {
    let (text, _) = connection_signal(
        &crate::ConnectionStatus::Connected,
        crate::i18n::UiLocale::ZhCn,
    );
    assert_eq!(text, "连接正常");
    let (text, _) = connection_signal(
        &crate::ConnectionStatus::Connecting,
        crate::i18n::UiLocale::EnUs,
    );
    assert_eq!(text, "Connecting");
    let (text, _) = connection_signal(
        &crate::ConnectionStatus::Error("x".to_string()),
        crate::i18n::UiLocale::EnUs,
    );
    assert_eq!(text, "Conn Error");
}

#[test]
fn connection_signal_uses_error_palette() {
    let (_, color) = connection_signal(
        &crate::ConnectionStatus::Error("failed".to_string()),
        crate::i18n::UiLocale::ZhCn,
    );
    assert_eq!(color, egui::Color32::from_rgb(160, 52, 52));
}

#[test]
fn health_signal_uses_three_levels() {
    let (ok_text, ok_color) = health_signal(0, crate::i18n::UiLocale::EnUs);
    assert_eq!(ok_text, "Health: OK");
    assert_eq!(ok_color, egui::Color32::from_rgb(32, 112, 64));

    let (warn_text, warn_color) = health_signal(2, crate::i18n::UiLocale::ZhCn);
    assert_eq!(warn_text, "健康:告警2");
    assert_eq!(warn_color, egui::Color32::from_rgb(150, 110, 32));

    let (high_text, high_color) = health_signal(3, crate::i18n::UiLocale::EnUs);
    assert_eq!(high_text, "Health: High 3");
    assert_eq!(high_color, egui::Color32::from_rgb(154, 48, 48));
}

#[test]
fn send_control_request_updates_playing_state() {
    let state = sample_viewer_state(crate::ConnectionStatus::Connected, Vec::new());
    let mut loading = crate::button_feedback::StepControlLoadingState::default();
    let mut control_ui = ControlPanelUiState::default();

    send_control_request(
        ViewerControl::Play,
        &state,
        &mut loading,
        &mut control_ui,
        None,
    );
    assert!(control_ui.playing);

    send_control_request(
        ViewerControl::Pause,
        &state,
        &mut loading,
        &mut control_ui,
        None,
    );
    assert!(!control_ui.playing);

    send_control_request(
        ViewerControl::Play,
        &state,
        &mut loading,
        &mut control_ui,
        None,
    );
    assert!(control_ui.playing);

    send_control_request(
        ViewerControl::Step { count: 1 },
        &state,
        &mut loading,
        &mut control_ui,
        None,
    );
    assert!(!control_ui.playing);
}

#[derive(Default)]
struct ControlButtonsHarnessState {
    viewer_state: crate::ViewerState,
    loading: crate::button_feedback::StepControlLoadingState,
    control_ui: ControlPanelUiState,
}

#[derive(Default)]
struct ThemeRuntimeHarnessState {
    runtime: crate::app_bootstrap::ThemeRuntimeState,
}

#[test]
fn egui_kittest_control_buttons_merge_play_pause_and_fold_advanced_debug() {
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut ControlButtonsHarnessState| {
            render_control_buttons(
                ui,
                crate::i18n::UiLocale::ZhCn,
                &state.viewer_state,
                &mut state.loading,
                &mut state.control_ui,
                None,
            );
        },
        ControlButtonsHarnessState::default(),
    );

    harness.fit_contents();
    harness.get_by_label("播放");
    harness.get_by_label("高级调试:关");

    harness.get_by_label("播放").click();
    harness.run();
    assert!(harness.state().control_ui.playing);
    harness.get_by_label("暂停");

    harness.get_by_label("高级调试:关").click();
    harness.run();
    assert!(harness.state().control_ui.advanced_debug_expanded);
    harness.get_by_label("单步");
    harness.get_by_label("跳转 0");
}

#[test]
fn egui_kittest_theme_runtime_apply_and_hot_reload_controls_work() {
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut ThemeRuntimeHarnessState| {
            super::egui_right_panel_theme_runtime::render_theme_runtime_section(
                ui,
                crate::i18n::UiLocale::ZhCn,
                &mut state.runtime,
            );
        },
        ThemeRuntimeHarnessState::default(),
    );

    harness.fit_contents();
    harness.get_by_label("应用主题").click();
    harness.run();
    assert!(harness.state().runtime.pending_apply);

    harness.get_by_label("自动热重载").click();
    harness.run();
    assert!(harness.state().runtime.hot_reload_enabled);
}

#[test]
fn mode_signal_reflects_timeline_state() {
    let live_timeline = TimelineUiState::default();
    let (live_text, live_color) = mode_signal(&live_timeline, crate::i18n::UiLocale::EnUs);
    assert_eq!(live_text, "View: Live");
    assert_eq!(live_color, egui::Color32::from_rgb(38, 94, 148));

    let manual_timeline = TimelineUiState {
        drag_active: true,
        ..Default::default()
    };
    let (manual_text, manual_color) = mode_signal(&manual_timeline, crate::i18n::UiLocale::ZhCn);
    assert_eq!(manual_text, "观察:手动");
    assert_eq!(manual_color, egui::Color32::from_rgb(125, 96, 28));
}

#[test]
fn feedback_tone_for_event_maps_warning_positive_and_info() {
    let warning = feedback_tone_for_event(&sample_rejected_event(1, 1).kind);
    assert_eq!(warning, FeedbackTone::Warning);

    let positive = feedback_tone_for_event(&sample_agent_moved_event(2, 2).kind);
    assert_eq!(positive, FeedbackTone::Positive);

    let info = feedback_tone_for_event(&WorldEventKind::LocationRegistered {
        location_id: "loc-1".to_string(),
        name: "alpha".to_string(),
        pos: agent_world::geometry::GeoPos::new(0.0, 0.0, 0.0),
        profile: Default::default(),
    });
    assert_eq!(info, FeedbackTone::Info);
}

#[test]
fn push_feedback_toast_clamps_queue_and_removes_oldest() {
    let mut feedback = FeedbackToastState::default();
    let locale = crate::i18n::UiLocale::EnUs;
    for id in 1..=(FEEDBACK_TOAST_MAX as u64 + 2) {
        push_feedback_toast(
            &mut feedback,
            &sample_rejected_event(id, id),
            10.0 + id as f64,
            locale,
        );
    }

    assert_eq!(feedback.toasts.len(), FEEDBACK_TOAST_MAX);
    let ids: Vec<u64> = feedback.toasts.iter().map(|toast| toast.id).collect();
    assert_eq!(ids, vec![3, 4, 5]);
}

#[test]
fn sync_feedback_toasts_skips_history_then_tracks_new_events_only() {
    let mut feedback = FeedbackToastState::default();
    let mut state = sample_viewer_state(
        crate::ConnectionStatus::Connected,
        vec![sample_rejected_event(1, 1), sample_agent_moved_event(2, 2)],
    );
    let locale = crate::i18n::UiLocale::ZhCn;

    sync_feedback_toasts(&mut feedback, &state, 20.0, locale);
    assert!(feedback.toasts.is_empty());
    assert_eq!(feedback.last_seen_event_id, Some(2));

    state.events.push(sample_rejected_event(3, 3));
    sync_feedback_toasts(&mut feedback, &state, 21.0, locale);

    assert_eq!(feedback.last_seen_event_id, Some(3));
    assert_eq!(feedback.toasts.len(), 1);
    assert_eq!(feedback.toasts[0].id, 3);
    assert_eq!(feedback.toasts[0].tone, FeedbackTone::Warning);
    assert_eq!(feedback.toasts[0].title, "操作受阻");
}

#[test]
fn truncate_observe_text_keeps_short_text() {
    let text = "观察";
    assert_eq!(truncate_observe_text(text, 8), text);
}

#[test]
fn truncate_observe_text_supports_multibyte_chars() {
    let text = "观察模式状态很长很长";
    let truncated = truncate_observe_text(text, 6);
    assert_eq!(truncated.chars().count(), 6);
    assert!(truncated.ends_with('…'));
}

#[test]
fn event_row_preview_limit_uses_constant() {
    let long_line = "x".repeat(EVENT_ROW_LABEL_MAX_CHARS + 20);
    let preview = truncate_observe_text(&long_line, EVENT_ROW_LABEL_MAX_CHARS);
    assert_eq!(preview.chars().count(), EVENT_ROW_LABEL_MAX_CHARS);
    assert!(preview.ends_with('…'));
}

#[test]
fn egui_kittest_overview_renders_status_badges() {
    let state = sample_viewer_state(crate::ConnectionStatus::Connected, Vec::new());
    let selection = crate::ViewerSelection::default();
    let timeline = TimelineUiState::default();

    let mut harness = Harness::new_ui(move |ui| {
        render_overview_section(
            ui,
            crate::i18n::UiLocale::ZhCn,
            &state,
            &selection,
            &timeline,
            None,
        );
    });

    harness.fit_contents();
    harness.get_by_label_contains("连接正常");
    harness.get_by_label_contains("健康:正常");
    harness.get_by_label_contains("观察:实时");
    harness.get_by_label_contains("状态: 已连接");
}

#[test]
fn egui_kittest_overview_reacts_to_warn_and_manual_mode() {
    let state = sample_viewer_state(
        crate::ConnectionStatus::Connected,
        vec![sample_rejected_event(1, 1)],
    );
    let selection = crate::ViewerSelection::default();
    let timeline = TimelineUiState {
        manual_override: true,
        ..Default::default()
    };

    let mut harness = Harness::new_ui(move |ui| {
        render_overview_section(
            ui,
            crate::i18n::UiLocale::EnUs,
            &state,
            &selection,
            &timeline,
            None,
        );
    });

    harness.fit_contents();
    harness.get_by_label_contains("Health: Warn 1");
    harness.get_by_label_contains("View: Manual");
    harness.get_by_label_contains("Status: connected");
}

#[test]
fn egui_kittest_overview_renders_render_perf_summary() {
    let state = sample_viewer_state(crate::ConnectionStatus::Connected, Vec::new());
    let selection = crate::ViewerSelection::default();
    let timeline = TimelineUiState::default();
    let perf = crate::RenderPerfSummary {
        frame_ms_avg: 16.7,
        frame_ms_p95: 24.2,
        world_entities: 180,
        visible_labels: 46,
        overlay_entities: 22,
        event_window_size: 88,
        auto_degrade_active: true,
    };

    let mut harness = Harness::new_ui(move |ui| {
        render_overview_section(
            ui,
            crate::i18n::UiLocale::EnUs,
            &state,
            &selection,
            &timeline,
            Some(&perf),
        );
    });

    harness.fit_contents();
    harness.get_by_label_contains("Render: avg/p95 16.7/24.2 ms");
    harness.get_by_label_contains("Entities:180");
    harness.get_by_label_contains("Budget: auto degrade active");
}

#[derive(Default)]
struct TimelineFilterHarnessState {
    viewer_state: crate::ViewerState,
    timeline: TimelineUiState,
    filters: TimelineMarkFilterState,
}

#[test]
fn egui_kittest_timeline_filter_button_toggles_state() {
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut TimelineFilterHarnessState| {
            render_timeline_section(
                ui,
                crate::i18n::UiLocale::ZhCn,
                &state.viewer_state,
                &mut state.timeline,
                &mut state.filters,
                None,
            );
        },
        TimelineFilterHarnessState::default(),
    );

    harness.fit_contents();
    harness.get_by_label("错误:开").click();
    harness.run();
    assert!(!harness.state().filters.show_error);

    harness.get_by_label("错误:关").click();
    harness.run();
    assert!(harness.state().filters.show_error);
}

#[test]
fn egui_kittest_overlay_section_renders_chunk_legend_and_width_hint() {
    let state = sample_viewer_state(crate::ConnectionStatus::Connected, Vec::new());
    let viewer_3d_config: Option<Res<crate::Viewer3dConfig>> = None;
    let mut overlay_config = crate::WorldOverlayConfig::default();

    let mut harness = Harness::new_ui(move |ui| {
        render_overlay_section(
            ui,
            crate::i18n::UiLocale::ZhCn,
            crate::ViewerCameraMode::TwoD,
            &state,
            &viewer_3d_config,
            &mut overlay_config,
        );
    });

    harness.fit_contents();
    harness.get_by_label_contains("分块图例");
    harness.get_by_label_contains("未探索");
    harness.get_by_label_contains("已生成");
    harness.get_by_label_contains("已耗尽");
    harness.get_by_label_contains("背景网格");
    harness.get_by_label_contains("线宽(2D)");
}

#[derive(Default)]
struct CameraModeHarnessState {
    camera_mode: crate::ViewerCameraMode,
}

#[test]
fn egui_kittest_camera_mode_toggle_switches_state() {
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut CameraModeHarnessState| {
            ui.horizontal_wrapped(|ui| {
                let is_two_d = state.camera_mode == crate::ViewerCameraMode::TwoD;
                if ui
                    .selectable_label(
                        is_two_d,
                        crate::i18n::camera_mode_button_label(
                            crate::ViewerCameraMode::TwoD,
                            crate::i18n::UiLocale::ZhCn,
                        ),
                    )
                    .clicked()
                {
                    state.camera_mode = crate::ViewerCameraMode::TwoD;
                }
                if ui
                    .selectable_label(
                        !is_two_d,
                        crate::i18n::camera_mode_button_label(
                            crate::ViewerCameraMode::ThreeD,
                            crate::i18n::UiLocale::ZhCn,
                        ),
                    )
                    .clicked()
                {
                    state.camera_mode = crate::ViewerCameraMode::ThreeD;
                }
            });
        },
        CameraModeHarnessState::default(),
    );

    harness.fit_contents();
    assert_eq!(harness.state().camera_mode, crate::ViewerCameraMode::TwoD);

    harness.get_by_label("3D").click();
    harness.run();
    assert_eq!(harness.state().camera_mode, crate::ViewerCameraMode::ThreeD);

    harness.get_by_label("2D").click();
    harness.run();
    assert_eq!(harness.state().camera_mode, crate::ViewerCameraMode::TwoD);
}

#[derive(Default)]
struct ModuleToggleHarnessState {
    visibility: RightPanelModuleVisibilityState,
    copyable_visible: bool,
}

#[test]
fn egui_kittest_module_toggle_switches_visibility() {
    let mut harness = Harness::new_ui_state(
        |ui, state: &mut ModuleToggleHarnessState| {
            ui.horizontal_wrapped(|ui| {
                render_module_toggle_button(
                    ui,
                    "controls",
                    &mut state.visibility.show_controls,
                    crate::i18n::UiLocale::ZhCn,
                );

                let mut details_visible = state.visibility.show_details;
                render_module_toggle_button(
                    ui,
                    "details",
                    &mut details_visible,
                    crate::i18n::UiLocale::ZhCn,
                );
                state.visibility.show_details = details_visible;
                state.copyable_visible = details_visible;
            });
        },
        ModuleToggleHarnessState {
            visibility: RightPanelModuleVisibilityState::default(),
            copyable_visible: true,
        },
    );

    harness.fit_contents();
    harness.get_by_label("控制:开").click();
    harness.run();
    assert!(!harness.state().visibility.show_controls);

    harness.get_by_label("明细:关").click();
    harness.run();
    assert!(harness.state().visibility.show_details);
    assert!(harness.state().copyable_visible);

    harness.get_by_label("明细:开").click();
    harness.run();
    assert!(!harness.state().visibility.show_details);
    assert!(!harness.state().copyable_visible);
}

#[test]
fn egui_kittest_snapshot_overview_live() {
    let Some(renderer) = snapshot_renderer_or_skip() else {
        return;
    };

    let state = sample_viewer_state(crate::ConnectionStatus::Connected, Vec::new());
    let selection = crate::ViewerSelection::default();
    let timeline = TimelineUiState::default();

    let mut harness = Harness::builder()
        .with_size(egui::vec2(380.0, 150.0))
        .renderer(renderer)
        .build_ui(move |ui| {
            render_overview_section(
                ui,
                crate::i18n::UiLocale::EnUs,
                &state,
                &selection,
                &timeline,
                None,
            );
        });

    harness.fit_contents();
    harness.snapshot_options("viewer_overview_live", &snapshot_options());
}

#[test]
fn egui_kittest_snapshot_overview_manual_high_risk() {
    let Some(renderer) = snapshot_renderer_or_skip() else {
        return;
    };

    let state = sample_viewer_state(
        crate::ConnectionStatus::Connected,
        vec![
            sample_rejected_event(1, 1),
            sample_rejected_event(2, 2),
            sample_rejected_event(3, 3),
        ],
    );
    let selection = crate::ViewerSelection::default();
    let timeline = TimelineUiState {
        manual_override: true,
        ..Default::default()
    };

    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 160.0))
        .renderer(renderer)
        .build_ui(move |ui| {
            render_overview_section(
                ui,
                crate::i18n::UiLocale::EnUs,
                &state,
                &selection,
                &timeline,
                None,
            );
        });

    harness.fit_contents();
    harness.snapshot_options("viewer_overview_manual_high_risk", &snapshot_options());
}

#[test]
fn rejection_event_count_only_counts_rejected_events() {
    use agent_world::geometry::GeoPos;

    let events = vec![
        WorldEvent {
            id: 1,
            time: 1,
            kind: WorldEventKind::LocationRegistered {
                location_id: "loc-1".to_string(),
                name: "Alpha".to_string(),
                pos: GeoPos::new(0.0, 0.0, 0.0),
                profile: Default::default(),
            },
        },
        WorldEvent {
            id: 2,
            time: 2,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::AgentNotFound {
                    agent_id: "a-1".to_string(),
                },
            },
        },
    ];

    assert_eq!(rejection_event_count(&events), 1);
}
