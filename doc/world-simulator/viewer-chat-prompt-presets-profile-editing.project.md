# Chat Panel 预设区扩展 Agent Prompt 字段编辑（项目管理文档）

## 任务拆解
- [x] VCPE1 输出设计文档（`doc/world-simulator/viewer-chat-prompt-presets-profile-editing.md`）
- [x] VCPE2 输出项目管理文档（本文件）
- [ ] VCPE3 在 Chat Panel 预设区新增 `system/short/long` 三字段编辑 UI
- [ ] VCPE4 接入 `prompt_control.apply` 提交链路（按选中 Agent）
- [ ] VCPE5 更新测试与手册，并执行 `test_tier_required` 回归
- [ ] VCPE6 回写状态与 devlog，完成收口提交

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world_viewer/src/main.rs`
- `doc/viewer-manual.md`
- `doc/world-simulator/viewer-chat-prompt-presets-profile-editing.md`

## 状态
- 当前阶段：VCPE1-VCPE2 已完成，VCPE3-VCPE6 待执行。
- 下一步：实现折叠区内 Agent Prompt Draft 子区与 apply 提交按钮。
- 最近更新：2026-02-16。
