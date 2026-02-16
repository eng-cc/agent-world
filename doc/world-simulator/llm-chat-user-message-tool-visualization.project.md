# LLM 用户文本输出与工具调用分区可视化（项目管理文档）

## 任务拆解
- [x] LUV1 输出设计文档（`doc/world-simulator/llm-chat-user-message-tool-visualization.md`）
- [x] LUV2 输出项目管理文档（本文件）
- [x] LUV3 LLM 解析协议扩展：支持 `message_to_user` 并保持向后兼容
- [x] LUV4 LLM 行为循环改造：仅记录用户可读 Agent 消息，不再记录原始 JSON
- [x] LUV5 Prompt schema/rules 更新，引导模型输出 `message_to_user`
- [x] LUV6 Viewer Chat 分区展示：信息流与工具调用流拆分
- [x] LUV7 工具调用可视化卡片化（模块名/状态/参数摘要/结果摘要）
- [x] LUV8 测试与回归（`test_tier_required`）
- [x] LUV9 文档回写、devlog 记录、收口提交

## 依赖
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/behavior_loop.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `crates/agent_world_viewer/src/egui_right_panel_chat.rs`

## 状态
- 当前阶段：已完成（LUV1-LUV9）
- 下一步：等待验收与后续迭代需求
- 最近更新：2026-02-16（LUV9 收口）
