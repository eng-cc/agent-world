# Agent World Runtime：Governance Tiered Offload 与 Rollback Audit 算术语义硬化（15 点清单第十阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase10.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase10.project.md`

### T1 Tiered Offload/Drill Alert 受检化
- [ ] `membership_recovery/replay_archive_tiered.rs` 的 offload/silence/cooldown 时间差改为受检减法。
- [ ] 去除 `plan_governance_audit_tiered_offload` 中不必要饱和计数语义。
- [ ] 补齐 underflow/overflow 拒绝测试并验证无状态污染。

### T2 Rollback Audit/Governance 受检化
- [ ] `membership_recovery/replay_audit.rs` 的 rollback streak 递进改为受检加法。
- [ ] rollback alert 窗口/cooldown 时间差改为受检减法。
- [ ] 补齐 overflow/underflow 测试并验证治理状态不被部分更新。

### T3 回归与收口
- [ ] 运行 `agent_world_consensus` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_consensus/src/membership_recovery/replay_archive_tiered.rs`
- `crates/agent_world_consensus/src/membership_recovery/replay_audit.rs`
- `crates/agent_world_consensus/src/membership_dead_letter_replay_archive_tests.rs`
- `crates/agent_world_consensus/src/membership_dead_letter_replay_audit_tests.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2、T3
- 阻塞项：无
