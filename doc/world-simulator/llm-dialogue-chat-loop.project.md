# Agent World Simulator：LLM 对话轮次驱动与右侧 Chat 面板（项目管理文档）

## 任务拆解
- [x] LDC1 输出设计文档（`doc/world-simulator/llm-dialogue-chat-loop.md`）
- [x] LDC2 输出项目管理文档（本文件）
- [ ] LDC3 LLM 决策从 step 状态机收敛为会话轮次循环（去 step prompt 元信息）
- [ ] LDC4 接入会话消息模型（玩家/Agent/工具/系统）与 trace 落盘
- [ ] LDC5 动作拒绝与工具结果回灌到后续 LLM 对话轮次
- [ ] LDC6 扩展 viewer 协议：`AgentChat` 请求与 ack/error 响应
- [ ] LDC7 live server 接入玩家消息注入 LLM Agent 会话
- [ ] LDC8 viewer 右侧新增 Chat 模块（选择 Agent、发送消息、消息列表）
- [ ] LDC9 回归测试（`test_tier_required` + 关键联调检查）
- [ ] LDC10 文档回写、devlog、收口提交

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/simulator/llm_agent/behavior_loop.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/agent.rs`
- `crates/agent_world_proto/src/viewer.rs`
- `crates/agent_world/src/viewer/live.rs`
- `crates/agent_world_viewer/src/egui_right_panel.rs`
- `crates/agent_world_viewer/src/right_panel_module_visibility.rs`

## 状态
- 当前阶段：LDC2（文档初始化完成）
- 下一步：LDC3-LDC5（先完成 LLM 会话内核改造与反馈回灌）
- 最近更新：完成设计与任务拆解（2026-02-16）
