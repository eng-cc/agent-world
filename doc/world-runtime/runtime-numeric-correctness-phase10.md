# Agent World Runtime：Governance Tiered Offload 与 Rollback Audit 算术语义硬化（15 点清单第十阶段）

## 目标
- 收口 `membership_recovery/replay_archive_tiered` 与 `membership_recovery/replay_audit` 中剩余的高风险饱和算术，避免长期运行下的静默数值失真。
- 将治理归档与告警窗口中的时间差计算从“饱和继续”改为“受检失败”，在时间回拨/边界异常时返回显式错误。
- 将 rollback 治理 streak 递进从饱和累加改为受检语义，避免 `usize::MAX` 长期运行边界下的等级判断错误。

## 范围

### In Scope（第十阶段）
- `crates/agent_world_consensus/src/membership_recovery/replay_archive_tiered.rs`
  - tiered offload 的 `now_ms - audited_at_ms` 改为受检减法。
  - recovery drill alert 的 cooldown/silence 时间差改为受检减法。
  - 计数器路径去除不必要的饱和减法/加法，改为确定性递减/递增。
- `crates/agent_world_consensus/src/membership_recovery/replay_audit.rs`
  - rollback governance `rollback_streak + 1` 改为受检递进。
  - rollback alert 的窗口与 cooldown 时间差改为受检减法。
- 测试：
  - 新增 overflow/underflow 拒绝测试。
  - 验证失败路径下治理状态、告警状态、归档存储不被部分污染。

### Out of Scope（后续阶段）
- 全仓库治理模块统一时间新类型（`Millis`/`BlockTime`）改造。
- 全链路 BigInt/U256 泛化替换。
- 治理策略产品语义重构（本阶段只做数值正确性）。

## 接口/数据
- 不改变对外 API 路径与调用入口；保持 `Result<..., WorldError>` 契约。
- 内部函数将引入受检返回：
  - `plan_governance_audit_tiered_offload` 改为可失败返回。
  - `evaluate_recovery_drill_alert_reasons` 改为可失败返回。
  - `apply_dead_letter_replay_rollback_governance_policy` 改为可失败返回。
- 统一错误语义：
  - `WorldError::DistributedValidationFailed`，错误消息包含字段上下文（`now_ms/last_alert_at_ms/audited_at_ms/rollback_streak`）。

## 里程碑
- M0：Phase10 建档并冻结边界。
- M1：tiered offload 与 drill alert 时间算术受检化完成。
- M2：rollback governance/alert 算术受检化与边界测试完成。
- M3：回归测试通过，文档/devlog 收口。

## 风险
- 行为会从“异常边界下继续执行”切换为“显式失败”，需要同步更新测试预期。
- 受检失败如果接线不完整，可能出现局部状态写入，需要重点验证“先算后写”。
- 归档/告警/治理三条路径耦合较高，需防止单点改动导致行为分叉。

## 当前状态
- 截至 2026-02-23：M0、M1、M2、M3 已完成（阶段收口）。
