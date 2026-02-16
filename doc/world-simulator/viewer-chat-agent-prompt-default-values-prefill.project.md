# Chat Panel Agent Prompt 默认值预填充输入框（项目管理文档）

## 任务拆解
- [x] VCPPF1 输出设计文档（`doc/world-simulator/viewer-chat-agent-prompt-default-values-prefill.md`）
- [x] VCPPF2 输出项目管理文档（本文件）
- [ ] VCPPF3 实现输入框默认值预填充与 patch 语义改造
- [ ] VCPPF4 更新测试与手册，执行 `test_tier_required` 回归
- [ ] VCPPF5 回写状态与 devlog，完成收口提交

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `doc/viewer-manual.md`
- `doc/world-simulator/viewer-chat-agent-prompt-default-values-prefill.md`

## 状态
- 当前阶段：VCPPF1-VCPPF2 已完成，VCPPF3-VCPPF5 待执行。
- 下一步：实现“无 override 时默认值填入输入框”并保证 apply 无误提交。
- 最近更新：2026-02-16。
