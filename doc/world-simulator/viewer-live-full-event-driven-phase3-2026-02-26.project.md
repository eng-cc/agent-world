# Viewer Live 完全事件驱动改造 Phase 3（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 LLM mailbox 事件化：`LlmDecisionRequested` + mailbox 计数
- [ ] T2 回归测试：mailbox 语义 + live 关键语义
- [ ] T3 文档与日志收口

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/consensus_bridge.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：T1 已完成，进行 T2
- 备注：Phase 3 聚焦 LLM 决策 mailbox 事件化；后续再评估跨模块总线统一与背压策略。
