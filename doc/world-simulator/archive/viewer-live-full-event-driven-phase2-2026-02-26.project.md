# Viewer Live 完全事件驱动改造 Phase 2（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 动态脉冲控制：暂停态零脉冲
- [x] T2 回归测试：脉冲启停 + live 关键语义
- [x] T3 文档与日志收口

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：已完成（T0~T3）
- 备注：Phase 2 收口完成，下一阶段进入 LLM mailbox 事件化与总线收敛。
