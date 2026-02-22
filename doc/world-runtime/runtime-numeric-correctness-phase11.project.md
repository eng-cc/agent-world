# Agent World Runtime：Membership Replay 调度/冷却时间门控算术语义硬化（15 点清单第十一阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase11.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase11.project.md`

### T1 调度与策略冷却时间门控受检化
- [ ] `run_revocation_dead_letter_replay_schedule_with_state_store` 的 interval gate 改为受检减法。
- [ ] `recommend_revocation_dead_letter_replay_policy_with_adaptive_guard` 的 policy cooldown 改为受检减法。
- [ ] 补齐对应 overflow 拒绝测试并验证 replay state 不被污染。

### T2 Rollback 冷却门控受检化
- [ ] `should_rollback_to_stable_policy` 的 rollback cooldown 改为受检减法并沿调用链显式失败。
- [ ] 补齐 rollback cooldown overflow 测试并验证 policy state 不被部分更新。

### T3 回归与收口
- [ ] 运行 `agent_world_consensus` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_consensus/src/membership_recovery/replay.rs`
- `crates/agent_world_consensus/src/membership_dead_letter_replay_tests.rs`
- `crates/agent_world_consensus/src/membership_dead_letter_replay_persistence_tests.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2、T3
- 阻塞项：无
