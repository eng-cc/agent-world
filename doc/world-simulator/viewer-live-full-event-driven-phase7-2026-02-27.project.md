# Viewer Live 完全事件驱动改造 Phase 7（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [ ] T1 script 节拍策略：`timer_pulse` / `event_drive` 双模接线
- [ ] T2 可观测性：信号级吞吐/merge/drop/处理耗时统计
- [ ] T3 回归测试 + Web 闭环验证
- [ ] T4 文档与日志收口

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：进行 T1
- 备注：Phase 7 聚焦 script 节拍策略与“完全事件驱动”验收指标。
