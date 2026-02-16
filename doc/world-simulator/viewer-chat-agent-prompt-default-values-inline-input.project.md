# Chat Panel Agent Prompt 默认值内联到输入框（项目管理文档）

## 任务拆解
- [x] VCPDII1 输出设计文档（`doc/world-simulator/viewer-chat-agent-prompt-default-values-inline-input.md`）
- [x] VCPDII2 输出项目管理文档（本文件）
- [x] VCPDII3 将默认值内联到三处输入框占位文本并删除单独提示行
- [x] VCPDII4 更新测试与手册，执行 `test_tier_required` 回归
- [x] VCPDII5 回写状态与 devlog，完成收口提交

## 依赖
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`
- `doc/viewer-manual.md`
- `doc/world-simulator/viewer-chat-agent-prompt-default-values-inline-input.md`

## 状态
- 当前阶段：VCPDII1-VCPDII5 已全部完成。
- 下一步：如需增强可发现性，可在字段右侧补一个“重置为默认”快捷按钮。
- 最近更新：2026-02-16（收口）。
