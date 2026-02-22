# Agent World Runtime：Governance Drill/Retention 时间算术数值语义硬化（15 点清单第九阶段）

## 目标
- 收口 `membership_recovery/replay_archive` 中治理审计保留与恢复演练调度的时间算术饱和语义。
- 将 `scheduled_at_ms +/- interval` 与 `now_ms - audited_at_ms` 从“静默饱和”改为“显式失败”，避免极端时间边界下误调度或误保留。
- 保持失败不污染状态：溢出时不写入 schedule state，不覆盖 audit retention 存储。

## 范围

### In Scope（第九阶段）
- `crates/agent_world_consensus/src/membership_recovery/replay_archive.rs`
  - recovery drill schedule：
    - `elapsed = scheduled_at_ms - last_drill_at_ms` 改为受检减法；
    - `next_due_at_ms = last_drill_at_ms + drill_interval_ms` 与 `scheduled_at_ms + drill_interval_ms` 改为受检加法。
  - governance audit retention：
    - `age = now_ms - audited_at_ms` 改为受检减法，拒绝溢出。
- 测试：
  - 新增 schedule `next_due_at_ms` 溢出拒绝测试；
  - 新增 retention `age` 计算溢出拒绝测试；
  - 验证溢出失败时状态不被部分更新。

### Out of Scope（后续阶段）
- 其他治理子模块全量时间算术统一改造。
- 全仓库时间戳 newtype 化与时钟源统一治理。
- 治理策略产品语义重构（仅处理数值正确性）。

## 接口/数据
- 不改变公开接口签名（均保持 `Result<..., WorldError>`）。
- 仅调整内部时间算术实现：
  - 溢出时统一返回 `WorldError::DistributedValidationFailed`，包含字段上下文。

## 里程碑
- M0：Phase9 建档并冻结范围。
- M1：drill schedule 时间算术受检语义落地。
- M2：audit retention 时间算术受检语义落地与测试补齐。
- M3：回归测试通过并完成文档/devlog 收口。

## 风险
- 由“饱和继续执行”切换为“显式失败”后，极端边界下行为预期会变化。
- 若只改部分路径，可能导致调度链条语义分裂，需要在单阶段内收口。
- 需确保错误返回发生在写入之前，避免半更新状态。

## 当前状态
- 截至 2026-02-23：M0 已完成；M1、M2、M3 进行中。
