use agent_world::simulator::WorldEvent;

use crate::i18n::{on_off_label, selection_kind_label, UiLocale};
use crate::{ConnectionStatus, SelectionKind, ViewerCameraMode, ViewerSelection};

pub(super) fn format_status_label(status: &ConnectionStatus, locale: UiLocale) -> String {
    if locale.is_zh() {
        match status {
            ConnectionStatus::Connecting => "连接中".to_string(),
            ConnectionStatus::Connected => "已连接".to_string(),
            ConnectionStatus::Error(message) => format!("错误: {message}"),
        }
    } else {
        match status {
            ConnectionStatus::Connecting => "connecting".to_string(),
            ConnectionStatus::Connected => "connected".to_string(),
            ConnectionStatus::Error(message) => format!("error: {message}"),
        }
    }
}

pub(super) fn status_line(status: &ConnectionStatus, locale: UiLocale) -> String {
    if locale.is_zh() {
        format!("状态: {}", format_status_label(status, locale))
    } else {
        format!("Status: {}", format_status_label(status, locale))
    }
}

pub(super) fn selection_line(selection: &ViewerSelection, locale: UiLocale) -> String {
    let Some(info) = selection.current.as_ref() else {
        return if locale.is_zh() {
            "选择: （无）".to_string()
        } else {
            "Selection: (none)".to_string()
        };
    };

    let kind = selection_kind_label(info.kind, locale);
    if locale.is_zh() {
        match info.kind {
            SelectionKind::Location => match &info.name {
                Some(name) => format!("选择: {kind} {} ({name})", info.id),
                None => format!("选择: {kind} {}", info.id),
            },
            _ => format!("选择: {kind} {}", info.id),
        }
    } else {
        match info.kind {
            SelectionKind::Location => match &info.name {
                Some(name) => format!("Selection: {kind} {} ({name})", info.id),
                None => format!("Selection: {kind} {}", info.id),
            },
            _ => format!("Selection: {kind} {}", info.id),
        }
    }
}

pub(super) fn summary_no_snapshot(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "世界: （无快照）"
    } else {
        "World: (no snapshot)"
    }
}

pub(super) fn agents_activity_no_snapshot(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "Agent 活动:\n（无快照）"
    } else {
        "Agents Activity:\n(no snapshot)"
    }
}

pub(super) fn details_click_to_inspect(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "详情:\n（点击对象查看）"
    } else {
        "Details:\n(click object to inspect)"
    }
}

pub(super) fn events_empty(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "事件:\n（无事件）"
    } else {
        "Events:\n(no events)"
    }
}

pub(super) fn diagnosis_waiting(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "诊断: 等待世界数据"
    } else {
        "Diagnosis: waiting world data"
    }
}

pub(super) fn event_links_waiting(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "事件联动:\n（等待事件）"
    } else {
        "Event Links:\n(waiting events)"
    }
}

pub(super) fn event_links_empty(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "事件联动:\n（无事件）"
    } else {
        "Event Links:\n(no events)"
    }
}

pub(super) fn event_links_hint(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "事件联动: 点击行定位对象"
    } else {
        "Event Links: click row to locate object"
    }
}

pub(super) fn link_ready(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "联动: 就绪"
    } else {
        "Link: ready"
    }
}

pub(super) fn locate_focus_label(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "定位焦点事件"
    } else {
        "Locate Focus"
    }
}

pub(super) fn jump_selection_label(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "跳转选中对象事件"
    } else {
        "Jump Selection"
    }
}

pub(super) fn overlay_loading(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "覆盖层: 加载中"
    } else {
        "Overlay: loading"
    }
}

pub(super) fn overlay_button_label(kind: &str, locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        match kind {
            "chunk" => "分块",
            "heat" => "热力",
            "flow" => "流向",
            "fragment" => "碎片",
            _ => "-",
        }
    } else {
        match kind {
            "chunk" => "Chunk",
            "heat" => "Heat",
            "flow" => "Flow",
            "fragment" => "Fragment",
            _ => "-",
        }
    }
}

pub(super) fn overlay_chunk_legend_title(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "分块图例"
    } else {
        "Chunk Legend"
    }
}

pub(super) fn overlay_chunk_legend_label(kind: &str, locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        match kind {
            "unexplored" => "未探索",
            "generated" => "已生成",
            "exhausted" => "已耗尽",
            "world_grid" => "背景网格",
            _ => "-",
        }
    } else {
        match kind {
            "unexplored" => "Unexplored",
            "generated" => "Generated",
            "exhausted" => "Exhausted",
            "world_grid" => "World Grid",
            _ => "-",
        }
    }
}

pub(super) fn overlay_grid_line_width_hint(
    locale: UiLocale,
    camera_mode: ViewerCameraMode,
    world_thickness: f32,
    chunk_thickness: f32,
) -> String {
    let mode = match camera_mode {
        ViewerCameraMode::TwoD => "2D",
        ViewerCameraMode::ThreeD => "3D",
    };
    if locale.is_zh() {
        format!("线宽({mode}): 背景={world_thickness:.3} 分块={chunk_thickness:.3}")
    } else {
        format!("Line Width ({mode}): world={world_thickness:.3} chunk={chunk_thickness:.3}")
    }
}

pub(super) fn overlay_status(
    snapshot: Option<(usize, usize, usize)>,
    heat_peak: Option<String>,
    flow_count: usize,
    show_chunk_overlay: bool,
    show_resource_heatmap: bool,
    show_flow_overlay: bool,
    locale: UiLocale,
) -> String {
    if locale.is_zh() {
        let mode = format!(
            "覆盖[分块:{} 热力:{} 流向:{}]",
            on_off_label(show_chunk_overlay, locale),
            on_off_label(show_resource_heatmap, locale),
            on_off_label(show_flow_overlay, locale)
        );
        let Some((unexplored, generated, exhausted)) = snapshot else {
            return format!("{mode} 无快照");
        };
        let heat_peak = heat_peak.unwrap_or_else(|| "-".to_string());
        return format!(
            "{mode} 分块(未探/已生/耗尽)={unexplored}/{generated}/{exhausted} 热力峰值={heat_peak} 流段={flow_count}"
        );
    }

    let mode = format!(
        "Overlay[chunk:{} heat:{} flow:{}]",
        on_off_label(show_chunk_overlay, locale),
        on_off_label(show_resource_heatmap, locale),
        on_off_label(show_flow_overlay, locale)
    );
    let Some((unexplored, generated, exhausted)) = snapshot else {
        return format!("{mode} no snapshot");
    };
    let heat_peak = heat_peak.unwrap_or_else(|| "-".to_string());
    format!(
        "{mode} chunks(u/g/e)={unexplored}/{generated}/{exhausted} heat_peak={heat_peak} flows={flow_count}"
    )
}

pub(super) fn timeline_status_line(
    current_tick: u64,
    target_tick: u64,
    axis_max: u64,
    mode_label: &str,
    locale: UiLocale,
) -> String {
    if locale.is_zh() {
        format!(
            "时间轴: 当前={} 目标={} 最大={} 模式={}",
            current_tick, target_tick, axis_max, mode_label
        )
    } else {
        format!(
            "Timeline: now={} target={} max={} mode={}",
            current_tick, target_tick, axis_max, mode_label
        )
    }
}

pub(super) fn timeline_mode_label(
    drag_active: bool,
    manual_override: bool,
    locale: UiLocale,
) -> &'static str {
    if locale.is_zh() {
        if drag_active {
            "拖拽"
        } else if manual_override {
            "手动"
        } else {
            "跟随"
        }
    } else if drag_active {
        "dragging"
    } else if manual_override {
        "manual"
    } else {
        "follow"
    }
}

pub(super) fn timeline_insights(
    error_len: usize,
    llm_len: usize,
    peak_len: usize,
    error_ticks: String,
    llm_ticks: String,
    peak_ticks: String,
    show_error: bool,
    show_llm: bool,
    show_peak: bool,
    sparkline: &str,
    locale: UiLocale,
) -> String {
    if locale.is_zh() {
        return format!(
            "标注: 错误={} LLM={} 峰值={}\n刻度: E[{}] L[{}] P[{}]\n过滤: 错误={} LLM={} 峰值={}\n密度: {}",
            error_len,
            llm_len,
            peak_len,
            error_ticks,
            llm_ticks,
            peak_ticks,
            on_off_label(show_error, locale),
            on_off_label(show_llm, locale),
            on_off_label(show_peak, locale),
            sparkline,
        );
    }

    format!(
        "Marks: err={} llm={} peak={}\nTicks: E[{}] L[{}] P[{}]\nFilter: err={} llm={} peak={}\nDensity: {}",
        error_len,
        llm_len,
        peak_len,
        error_ticks,
        llm_ticks,
        peak_ticks,
        on_off_label(show_error, locale),
        on_off_label(show_llm, locale),
        on_off_label(show_peak, locale),
        sparkline,
    )
}

pub(super) fn timeline_mark_filter_label(kind: &str, enabled: bool, locale: UiLocale) -> String {
    if locale.is_zh() {
        let prefix = match kind {
            "err" => "错误",
            "llm" => "LLM",
            "peak" => "峰值",
            _ => "-",
        };
        return format!("{}:{}", prefix, if enabled { "开" } else { "关" });
    }

    let prefix = match kind {
        "err" => "Err",
        "llm" => "LLM",
        "peak" => "Peak",
        _ => "-",
    };
    format!("{}:{}", prefix, if enabled { "ON" } else { "OFF" })
}

pub(super) fn timeline_jump_label(kind: &str, locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        match kind {
            "err" => "跳转错误",
            "llm" => "跳转LLM",
            "peak" => "跳转峰值",
            _ => "-",
        }
    } else {
        match kind {
            "err" => "Jump Err",
            "llm" => "Jump LLM",
            "peak" => "Jump Peak",
            _ => "-",
        }
    }
}

pub(super) fn seek_button_label(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "跳转"
    } else {
        "Seek"
    }
}

pub(super) fn map_link_message_for_locale(message: &str, locale: UiLocale) -> String {
    if !locale.is_zh() {
        return message.to_string();
    }

    let mut converted = message.to_string();
    converted = converted.replace("Link:", "联动:");
    converted = converted.replace("ready", "就绪");
    converted = converted.replace("no events available", "当前无事件");
    converted = converted.replace("has no mappable object", "没有可映射对象");
    converted = converted.replace("has no mappable target", "没有可映射目标");
    converted = converted.replace("target", "目标");
    converted = converted.replace("is not in current scene", "不在当前场景");
    converted = converted.replace("no selection", "当前无选择");
    converted = converted.replace("has no related events", "没有相关事件");
    converted = converted.replace("no target tick", "没有可跳转 tick");
    converted = converted.replace("event", "事件");
    converted = converted.replace("location", "地点");
    converted = converted.replace("asset", "资产");
    converted = converted.replace("chunk", "分块");
    converted
}

pub(super) fn localize_world_summary_block(text: String, locale: UiLocale) -> String {
    if !locale.is_zh() {
        return text;
    }
    let mut converted = text;
    converted = converted.replace("World: (no snapshot)", "世界: （无快照）");
    converted = converted.replace("Time:", "时间:");
    converted = converted.replace("Locations:", "地点数:");
    converted = converted.replace("Agents:", "Agent 数:");
    converted = converted.replace("Assets:", "资产数:");
    converted = converted.replace("Module Visuals:", "模块可视实体:");
    converted = converted.replace("Power Plants:", "电厂数:");
    converted = converted.replace("Power Storages:", "储能数:");
    converted = converted.replace("Chunks:", "分块数:");
    converted = converted.replace("Ticks:", "Tick:");
    converted = converted.replace("Actions:", "动作数:");
    converted = converted.replace("Decisions:", "决策数:");
    converted = converted.replace("Render Physical: on", "物理渲染: 开启");
    converted = converted.replace("Render Physical: off", "物理渲染: 关闭");
    converted = converted.replace("Unit:", "单位:");
    converted = converted.replace("Camera Clip(m):", "相机裁剪(m):");
    converted = converted.replace("Stellar Distance(AU):", "恒星距离(AU):");
    converted = converted.replace("Irradiance(W/m²):", "辐照度(W/m²):");
    converted = converted.replace("Exposed Illuminance(lux):", "曝光后照度(lux):");
    converted = converted.replace("Exposure(EV100):", "曝光(EV100):");
    converted = converted.replace("Radiation Ref Area(m²):", "辐射参考面积(m²):");
    converted
}

pub(super) fn localize_events_summary_block(text: String, locale: UiLocale) -> String {
    if !locale.is_zh() {
        return text;
    }
    let mut converted = text;
    converted = converted.replace("Events (focused):", "事件（焦点）:");
    converted = converted.replace("Events:\n(no events)", "事件:\n（无事件）");
    converted = converted.replace("Events:", "事件:");
    converted = converted.replace("Focus: requested", "焦点: 请求");
    converted = converted.replace(" -> nearest", " -> 最近");
    converted
}

pub(super) fn localize_agent_activity_block(text: String, locale: UiLocale) -> String {
    if !locale.is_zh() {
        return text;
    }
    let mut converted = text;
    converted = converted.replace("Agents Activity:\n(no snapshot)", "Agent 活动:\n（无快照）");
    converted = converted.replace("Agents Activity:\n(none)", "Agent 活动:\n（无）");
    converted = converted.replace("Agents Activity:", "Agent 活动:");
    converted = converted.replace("idle", "空闲");
    converted
}

pub(super) fn localize_details_block(text: String, locale: UiLocale) -> String {
    if !locale.is_zh() {
        return text;
    }
    let mut converted = text;
    converted = converted.replace(
        "Details:\n(click object to inspect)",
        "详情:\n（点击对象查看）",
    );
    converted = converted.replace("(no snapshot)", "（无快照）");
    converted = converted.replace("(not found in snapshot)", "（快照中未找到）");
    converted = converted.replace("(invalid chunk id)", "（分块 id 无效）");
    converted = converted.replace("(none)", "（无）");
    converted = converted.replace("(no llm trace yet)", "（暂无 LLM 轨迹）");
    converted = converted.replace("Details:", "详情:");
    converted = converted.replace("Location:", "地点:");
    converted = converted.replace("Body Size:", "机体尺寸:");
    converted = converted.replace("Location Radius:", "地点半径:");
    converted = converted.replace("Scale Ratio:", "尺度比例:");
    converted = converted.replace("Name:", "名称:");
    converted = converted.replace("Resources:", "资源:");
    converted = converted.replace("Recent Events:", "近期事件:");
    converted = converted.replace("Recent LLM I/O:", "近期 LLM 输入输出:");
    converted = converted.replace("Recent Owner Events:", "近期归属者事件:");
    converted = converted.replace("Budget (remaining top):", "预算（剩余 Top）:");
    converted = converted.replace("Budget (total top):", "预算（总量 Top）:");
    converted = converted.replace("Thermal Visual:", "热态可视:");
    converted = converted.replace("Radiation Visual:", "辐射可视:");
    converted = converted.replace("color=heat_low", "颜色=低温");
    converted = converted.replace("color=heat_mid", "颜色=暖态");
    converted = converted.replace("color=heat_high", "颜色=过热");
    converted
}

pub(super) fn localize_diagnosis_text(text: String, locale: UiLocale) -> String {
    if !locale.is_zh() {
        return text;
    }

    let mut converted = text;
    converted = converted.replace("Diagnosis:", "诊断:");
    converted = converted.replace("Conclusion:", "结论:");
    converted = converted.replace("viewer disconnected", "viewer 连接断开");
    converted = converted.replace("data stream unavailable", "数据流不可用");
    converted = converted.replace("check live server/network", "请检查 live server/网络");
    converted = converted.replace("LLM call failed", "LLM 调用失败");
    converted = converted.replace("decision degraded", "决策已降级");
    converted = converted.replace("check model endpoint/config", "请检查模型端点/配置");
    converted = converted.replace("decision parse failed", "决策解析失败");
    converted = converted.replace("model output format mismatch", "模型输出格式不匹配");
    converted = converted.replace("action rejected", "动作被拒绝");
    converted = converted.replace("no snapshot yet", "尚无快照");
    converted = converted.replace("wait for first world snapshot", "等待首帧世界快照");
    converted = converted.replace("no blocking issue detected", "未发现阻塞性问题");
    converted = converted.replace("simulation healthy", "模拟健康");
    converted = converted.replace("focus on selected", "可重点关注已选中");
    converted = converted.replace("resource shortage", "资源不足");
    converted = converted.replace("location constraints not satisfied", "位置约束不满足");
    converted = converted.replace("thermal overload", "热过载");
    converted = converted.replace("agent is shutdown", "Agent 已关机");
    converted = converted.replace("power transfer distance exceeded", "电力传输距离超限");
    converted = converted.replace(
        "power transfer loss exceeds amount",
        "电力传输损耗超过传输量",
    );
    converted = converted.replace("action preconditions not satisfied", "动作前置条件不满足");
    converted
}

pub(super) fn localized_event_row_label(
    event: &WorldEvent,
    focused: bool,
    locale: UiLocale,
) -> String {
    let mut body = format!("#{:>3} t{:>4} {:?}", event.id, event.time, event.kind);
    if locale.is_zh() {
        body = body.replace("ActionRejected", "动作拒绝");
        body = body.replace("LocationRegistered", "地点注册");
        body = body.replace("AgentMoved", "Agent 移动");
    }

    if focused {
        format!(">> {body}")
    } else {
        format!("   {body}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_line_supports_zh() {
        let text = status_line(&ConnectionStatus::Connected, UiLocale::ZhCn);
        assert_eq!(text, "状态: 已连接");
    }

    #[test]
    fn timeline_filter_label_supports_zh() {
        assert_eq!(
            timeline_mark_filter_label("err", true, UiLocale::ZhCn),
            "错误:开"
        );
    }

    #[test]
    fn overlay_grid_line_width_hint_supports_zh() {
        let text = overlay_grid_line_width_hint(UiLocale::ZhCn, ViewerCameraMode::TwoD, 0.01, 0.02);
        assert!(text.contains("线宽(2D)"));
        assert!(text.contains("背景=0.010"));
        assert!(text.contains("分块=0.020"));
    }
}
