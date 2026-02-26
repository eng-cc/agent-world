# Viewer Live 完全事件驱动改造 Phase 2（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [ ] T1 动态脉冲控制：暂停态零脉冲
- [ ] T2 回归测试：脉冲启停 + live 关键语义
- [ ] T3 文档与日志收口

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：T0 已完成，进行 T1
- 备注：Phase 2 目标是去除“暂停态空脉冲”；后续 Phase 3 再推进 LLM mailbox 事件化与总线收敛。
