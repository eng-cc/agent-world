# [已归档] Chat Panel Agent Prompt 字段默认值可见化（项目管理文档）

> 归档标记（2026-02-17）：该任务对应设计已被后续方案替代。
> 过时原因：任务口径为“默认值单独文案展示”，当前实现已迁移为预填充/内联输入方案，不再保留该展示形态。
> 替代文档：`doc/world-simulator/viewer-chat-agent-prompt-default-values-prefill.project.md`、`doc/world-simulator/viewer-chat-agent-prompt-default-values-inline-input.project.md`。

## 任务拆解
- [x] VCPDV1 输出设计文档（`doc/world-simulator/viewer-chat-agent-prompt-default-values.md`）
- [x] VCPDV2 输出项目管理文档（本文件）
- [x] VCPDV3 在 Chat Panel 三字段增加默认值展示
- [x] VCPDV4 补充测试与手册说明，执行 `test_tier_required` 回归
- [x] VCPDV5 回写状态与 devlog，完成收口提交

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `crates/agent_world/src/simulator/mod.rs`
- `doc/viewer-manual.md`
- `doc/world-simulator/viewer-chat-agent-prompt-default-values.md`

## 状态
- 当前阶段：VCPDV1-VCPDV5 已全部完成。
- 下一步：如需进一步降低误解，可在默认值文案旁增加“当前是否覆盖中”状态标记。
- 最近更新：2026-02-16（收口）。
