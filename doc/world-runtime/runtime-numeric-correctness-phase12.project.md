# Agent World Runtime：Membership Recovery 调度门控与计数聚合算术语义硬化（15 点清单第十二阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase12.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase12.project.md`

### T1 调度时间门控受检化
- [ ] `run_revocation_dead_letter_replay_schedule` 的 interval gate 改为受检减法。
- [ ] 新增 overflow 拒绝测试并验证 `last_replay_at_ms` 不被污染。

### T2 Recovery 计数/容量受检化
- [ ] `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter` 的关键 `saturating_add` 改为受检语义。
- [ ] 补齐计数边界 overflow 测试并验证 pending/dead-letter 不被部分更新。

### T3 回归与收口
- [ ] 运行 `agent_world_consensus` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_consensus/src/membership_recovery/mod.rs`
- `crates/agent_world_consensus/src/membership_recovery_tests.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2、T3
- 阻塞项：无
