# Viewer Chat 预设 Prompt 编辑区（项目管理文档）

## 任务拆解
- [x] VCP1 输出设计文档（`doc/world-simulator/viewer-chat-prompt-presets.md`）
- [x] VCP2 输出项目管理文档（本文件）
- [ ] VCP3 删除 Prompt Ops 模块入口与面板接线
- [ ] VCP4 在最右侧 Chat Panel 增加可展开预设 Prompt 编辑区（新增/编辑/删除/填充）
- [ ] VCP5 更新测试并执行回归（`test_tier_required`）
- [ ] VCP6 回写文档状态与 devlog，完成收口提交

## 依赖
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/app_bootstrap.rs`
- `crates/agent_world_viewer/src/i18n.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world_viewer/src/egui_right_panel_tests.rs`
- `doc/viewer-manual.md`

## 状态
- 当前阶段：VCP1-VCP2 已完成，VCP3-VCP6 待执行。
- 下一步：进入代码改造（先移除 Prompt Ops，再接入 Chat 预设编辑区）。
- 最近更新：2026-02-16。
