# Chat Panel Agent Prompt 字段默认值可见化（项目管理文档）

## 任务拆解
- [x] VCPDV1 输出设计文档（`doc/world-simulator/viewer-chat-agent-prompt-default-values.md`）
- [x] VCPDV2 输出项目管理文档（本文件）
- [ ] VCPDV3 在 Chat Panel 三字段增加默认值展示
- [ ] VCPDV4 补充测试与手册说明，执行 `test_tier_required` 回归
- [ ] VCPDV5 回写状态与 devlog，完成收口提交

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world/src/simulator/mod.rs`
- `doc/viewer-manual.md`
- `doc/world-simulator/viewer-chat-agent-prompt-default-values.md`

## 状态
- 当前阶段：VCPDV1-VCPDV2 已完成，VCPDV3-VCPDV5 待执行。
- 下一步：实现 `system/short/long` 默认值可见化，保持 override 编辑语义不变。
- 最近更新：2026-02-16。
