use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherUiFieldKind {
    Text,
    Checkbox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LauncherUiField {
    pub id: &'static str,
    pub section: &'static str,
    pub kind: LauncherUiFieldKind,
    pub label_zh: &'static str,
    pub label_en: &'static str,
    pub web_visible: bool,
    pub native_visible: bool,
}

const LAUNCHER_UI_FIELDS: &[LauncherUiField] = &[
    LauncherUiField {
        id: "scenario",
        section: "game_core",
        kind: LauncherUiFieldKind::Text,
        label_zh: "场景",
        label_en: "Scenario",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "live_bind",
        section: "game_core",
        kind: LauncherUiFieldKind::Text,
        label_zh: "实时服务绑定",
        label_en: "Live Bind",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "web_bind",
        section: "game_core",
        kind: LauncherUiFieldKind::Text,
        label_zh: "WebSocket 绑定",
        label_en: "Web Bind",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "viewer_host",
        section: "viewer_core",
        kind: LauncherUiFieldKind::Text,
        label_zh: "游戏页面主机",
        label_en: "Viewer Host",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "viewer_port",
        section: "viewer_core",
        kind: LauncherUiFieldKind::Text,
        label_zh: "游戏页面端口",
        label_en: "Viewer Port",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "llm_enabled",
        section: "viewer_core",
        kind: LauncherUiFieldKind::Checkbox,
        label_zh: "启用 LLM",
        label_en: "Enable LLM",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "chain_enabled",
        section: "viewer_core",
        kind: LauncherUiFieldKind::Checkbox,
        label_zh: "启用链运行时",
        label_en: "Enable Chain Runtime",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "auto_open_browser",
        section: "viewer_core",
        kind: LauncherUiFieldKind::Checkbox,
        label_zh: "自动打开浏览器",
        label_en: "Open Browser Automatically",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "chain_status_bind",
        section: "chain_identity",
        kind: LauncherUiFieldKind::Text,
        label_zh: "链状态服务绑定",
        label_en: "Chain Status Bind",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "chain_node_id",
        section: "chain_identity",
        kind: LauncherUiFieldKind::Text,
        label_zh: "链节点 ID",
        label_en: "Chain Node ID",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "chain_world_id",
        section: "chain_identity",
        kind: LauncherUiFieldKind::Text,
        label_zh: "链世界 ID",
        label_en: "Chain World ID",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "chain_node_role",
        section: "chain_runtime",
        kind: LauncherUiFieldKind::Text,
        label_zh: "链节点角色",
        label_en: "Chain Role",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "chain_node_tick_ms",
        section: "chain_runtime",
        kind: LauncherUiFieldKind::Text,
        label_zh: "链 Tick 毫秒",
        label_en: "Chain Tick Milliseconds",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "chain_node_validators",
        section: "chain_runtime",
        kind: LauncherUiFieldKind::Text,
        label_zh: "链验证者",
        label_en: "Chain Validators",
        web_visible: true,
        native_visible: true,
    },
    LauncherUiField {
        id: "launcher_bin",
        section: "binaries",
        kind: LauncherUiFieldKind::Text,
        label_zh: "启动器二进制路径",
        label_en: "Launcher Binary",
        web_visible: false,
        native_visible: true,
    },
    LauncherUiField {
        id: "chain_runtime_bin",
        section: "binaries",
        kind: LauncherUiFieldKind::Text,
        label_zh: "链运行时二进制路径",
        label_en: "Chain Runtime Binary",
        web_visible: false,
        native_visible: true,
    },
    LauncherUiField {
        id: "viewer_static_dir",
        section: "static_assets",
        kind: LauncherUiFieldKind::Text,
        label_zh: "前端静态资源目录",
        label_en: "Viewer Static Directory",
        web_visible: true,
        native_visible: true,
    },
];

pub fn launcher_ui_fields() -> &'static [LauncherUiField] {
    LAUNCHER_UI_FIELDS
}

pub fn launcher_ui_fields_for_web() -> impl Iterator<Item = &'static LauncherUiField> {
    LAUNCHER_UI_FIELDS.iter().filter(|field| field.web_visible)
}

pub fn launcher_ui_fields_for_native() -> impl Iterator<Item = &'static LauncherUiField> {
    LAUNCHER_UI_FIELDS
        .iter()
        .filter(|field| field.native_visible)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_fields_have_unique_ids() {
        let mut ids = std::collections::BTreeSet::new();
        for field in launcher_ui_fields() {
            assert!(ids.insert(field.id), "duplicate field id: {}", field.id);
        }
    }

    #[test]
    fn web_fields_exclude_native_only_binaries() {
        let ids: std::collections::BTreeSet<&str> =
            launcher_ui_fields_for_web().map(|field| field.id).collect();
        assert!(!ids.contains("launcher_bin"));
        assert!(!ids.contains("chain_runtime_bin"));
        assert!(ids.contains("scenario"));
        assert!(ids.contains("viewer_static_dir"));
    }
}
