# Viewer Live 完全事件驱动改造 Phase 2（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 动态脉冲控制：暂停态零脉冲
- [x] T2 回归测试：脉冲启停 + live 关键语义
- [ ] T3 文档与日志收口

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：T0/T1/T2 已完成，进行 T3
- 备注：已新增“disabled 无脉冲 / enabled 后发脉冲”回归用例并跑通 live 测试组；进入收口阶段。
