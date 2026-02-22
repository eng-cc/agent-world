# Agent World Runtime：Membership Reconciliation 调度门控与对账计数算术语义硬化（15 点清单第十三阶段）项目管理文档

## 任务拆解

### T0 建档
- [x] 新建设计文档：`doc/world-runtime/runtime-numeric-correctness-phase13.md`
- [x] 新建项目管理文档：`doc/world-runtime/runtime-numeric-correctness-phase13.project.md`

### T1 调度与 dedup 时间门控受检化
- [ ] `schedule_due` 的时间差从 `saturating_sub` 改为受检减法并透传错误。
- [ ] `deduplicate_revocation_alerts` 的 suppress window 时间差从 `saturating_sub` 改为受检减法。
- [ ] 增加 overflow 拒绝测试并验证 `schedule_state` / `dedup_state` 不被污染。

### T2 对账报告计数受检化
- [ ] `reconcile_revocations_with_policy` 的关键计数累加改为受检语义。
- [ ] 增加计数 helper overflow 测试并验证错误语义一致。

### T3 回归与收口
- [ ] 运行 `agent_world_consensus` 定向回归测试。
- [ ] 回写设计文档状态（M0~M3）。
- [ ] 回写项目状态与 `doc/devlog/2026-02-23.md`。

## 依赖
- `crates/agent_world_consensus/src/membership_reconciliation.rs`
- `crates/agent_world_consensus/src/membership_reconciliation_tests.rs`

## 状态
- 当前状态：`进行中`
- 已完成：T0
- 进行中：T1
- 未开始：T2、T3
- 阻塞项：无
