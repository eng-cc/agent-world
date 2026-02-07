# Viewer UI 多语言支持（中文 / 英文）（项目管理文档）

## 任务拆解
- [x] 输出设计文档（`doc/world-simulator/viewer-i18n.md`）
- [x] 输出项目管理文档（本文件）
- [x] I18N1：新增 `UiLocale` 与 UI 语言切换状态（默认 `zh-CN`）
- [x] I18N2：新增 `UiI18n` 资源与统一文本 key 枚举
- [x] I18N3：落地 `zh-CN` / `en-US` 双词典与回退链路
- [x] I18N4：迁移 `main.rs` 固定文案到 i18n key
- [x] I18N5：迁移 `ui_text.rs` 动态模板到 `format(key, args)`
- [x] I18N6：迁移 `diagnosis.rs` / `timeline_controls.rs` / `panel_layout.rs` 文案
- [x] I18N7：迁移 `event_click_list.rs` / `selection_linking.rs` / `world_overlay.rs` 文案
- [x] I18N8：新增/更新测试（UI 切换、词典回退、关键 UI 中英输出）
- [x] I18N9：更新总可视化文档与项目管理文档状态
- [x] I18N10：更新任务日志并提交

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/ui_text.rs`
- `crates/agent_world_viewer/src/diagnosis.rs`
- `crates/agent_world_viewer/src/timeline_controls.rs`
- `crates/agent_world_viewer/src/panel_layout.rs`
- `crates/agent_world_viewer/src/event_click_list.rs`
- `crates/agent_world_viewer/src/selection_linking.rs`
- `crates/agent_world_viewer/src/world_overlay.rs`

## 状态
- 当前阶段：I18N10 完成（中英切换实现 + 文档收口）
- 下一阶段：按需扩展第三语言与本地配置持久化
- 最近更新：完成 UI 多语言任务收口并提交前校验（2026-02-07）
