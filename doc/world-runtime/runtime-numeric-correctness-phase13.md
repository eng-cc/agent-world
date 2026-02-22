# Agent World Runtime：Membership Reconciliation 调度门控与对账计数算术语义硬化（15 点清单第十三阶段）

## 目标
- 收口 `membership_reconciliation.rs` 中剩余的高风险时间门控与报告计数饱和算术。
- 在极端时间边界下，将“静默饱和继续执行”升级为“显式失败且不污染状态”。
- 在对账报告聚合路径上统一受检计数语义，避免长期运行中的静默夹逼导致观测失真。

## 范围

### In Scope（第十三阶段）
- `crates/agent_world_consensus/src/membership_reconciliation.rs`
  - `schedule_due` 时间差从 `saturating_sub` 改为受检减法并透传错误。
  - `deduplicate_revocation_alerts` 的 suppress window 时间差从 `saturating_sub` 改为受检减法。
  - `reconcile_revocations_with_policy` 报告计数（`rejected/in_sync/diverged/merged`）从饱和累加改为受检累加。
  - 统一溢出错误语义为 `WorldError::DistributedValidationFailed`，错误消息包含关键现场值。
- 测试
  - 在 `membership_reconciliation_tests.rs` 增加时间差溢出拒绝测试，验证 `schedule_state` / `dedup_state` 不被污染。
  - 在模块内增加受检计数 helper 的 overflow 单元测试。

### Out of Scope（后续阶段）
- `membership.rs`、`mempool.rs`、`agent_world_node` 中尚未收口的计数饱和语义。
- `membership_recovery/replay*.rs` 非本阶段增量（已收口路径除外）。
- 全仓库数值 newtype 统一与跨模块时间源治理。

## 接口/数据
- 对外 API 入口保持不变（继续返回 `Result<..., WorldError>`）。
- 内部引入受检计数 helper（`checked_usize_add` / `checked_usize_increment`）替换关键 `saturating_add`。
- 失败模型统一：
  - `WorldError::DistributedValidationFailed`
  - 错误消息包含 `now_ms/last_run_ms` 或计数字段上下文。

## 里程碑
- M0：Phase13 建档并冻结边界。
- M1：调度门控与 dedup 时间门控受检改造完成。
- M2：对账报告计数受检改造与边界测试完成。
- M3：回归测试通过并完成文档/devlog 收口。

## 风险
- 从饱和语义切换到显式失败后，历史边界行为会从“继续执行”改为“拒绝”，需要同步更新测试预期。
- `schedule_due` 返回语义由布尔升级为可失败后，调用链若漏改会造成行为回退。
- 需重点保证失败路径不污染状态（尤其 `last_checkpoint_at_ms`、`last_reconcile_at_ms` 与 dedup state）。

## 当前状态
- 截至 2026-02-23：M0 已完成；M1、M2、M3 待执行。
