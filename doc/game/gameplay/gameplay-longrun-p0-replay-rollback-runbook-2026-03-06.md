# Gameplay Long-Run P0：Replay/Rollback 运行手册（2026-03-06）

审计轮次: 5

- 关联 PRD：`doc/game/gameplay/gameplay-longrun-p0-production-hardening-2026-03-06.prd.md`
- 覆盖任务：`TASK-GAME-014`（`PRD-GAME-006-02`）

## 1. 触发条件
- `verify_tick_consensus_chain()` 返回 `DistributedValidationFailed`。
- `first_tick_consensus_drift()` 返回非空（可定位 `mismatch_tick`）。
- 长稳门禁出现共识链路漂移告警，需要执行状态恢复演练或实战回滚。

## 2. 标准处置流程（Runbook）
1. 漂移定位：调用 `first_tick_consensus_drift()` 获取首个 `mismatch_tick` 与原因。
2. 影响区间确认：锁定最近稳定 `snapshot` 与对应 `journal`。
3. 回滚恢复：执行 `rollback_to_snapshot_with_reconciliation(snapshot, journal, reason)`。
4. 恢复对账：再次执行 `first_tick_consensus_drift()` 与 `verify_tick_consensus_chain()`，必须均为“无漂移”。
5. 审计留痕：确认 `RollbackApplied` 事件已写入事件链。

## 3. 演练命令（required-tier）
```bash
env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::persistence::rollback_with_reconciliation_recovers_from_detected_tick_consensus_drift -- --nocapture
```

## 4. 通过标准
- 演练命令返回 `rc=0`。
- 漂移被成功定位到具体 `mismatch_tick`。
- 回滚后 `first_tick_consensus_drift() == None`。
- 回滚后 `verify_tick_consensus_chain()` 通过。
- 事件链存在 `RollbackApplied` 记录。

## 5. 失败处置
- 若回滚后仍有漂移：立即阻断发布，保留快照/日志，升级到治理应急流程。
- 若漂移定位失败：先执行一次完整快照恢复重放，再人工比对 `tick_consensus_records` 链路。
