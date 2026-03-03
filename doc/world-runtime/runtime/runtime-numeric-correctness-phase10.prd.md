# Agent World Runtime：Governance Tiered Offload 与 Rollback Audit 算术语义硬化（15 点清单第十阶段）

- 对应项目管理文档: doc/world-runtime/runtime/runtime-numeric-correctness-phase10.prd.project.md

## 1. Executive Summary
- 收口 `membership_recovery/replay_archive_tiered` 与 `membership_recovery/replay_audit` 中剩余的高风险饱和算术，避免长期运行下的静默数值失真。
- 将治理归档与告警窗口中的时间差计算从“饱和继续”改为“受检失败”，在时间回拨/边界异常时返回显式错误。
- 将 rollback 治理 streak 递进从饱和累加改为受检语义，避免 `usize::MAX` 长期运行边界下的等级判断错误。

## 2. User Experience & Functionality
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


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- 不改变对外 API 路径与调用入口；保持 `Result<..., WorldError>` 契约。
- 内部函数将引入受检返回：
  - `plan_governance_audit_tiered_offload` 改为可失败返回。
  - `evaluate_recovery_drill_alert_reasons` 改为可失败返回。
  - `apply_dead_letter_replay_rollback_governance_policy` 改为可失败返回。
- 统一错误语义：
  - `WorldError::DistributedValidationFailed`，错误消息包含字段上下文（`now_ms/last_alert_at_ms/audited_at_ms/rollback_streak`）。

## 5. Risks & Roadmap
- M0：Phase10 建档并冻结边界。
- M1：tiered offload 与 drill alert 时间算术受检化完成。
- M2：rollback governance/alert 算术受检化与边界测试完成。
- M3：回归测试通过，文档/devlog 收口。

### Technical Risks
- 行为会从“异常边界下继续执行”切换为“显式失败”，需要同步更新测试预期。
- 受检失败如果接线不完整，可能出现局部状态写入，需要重点验证“先算后写”。
- 归档/告警/治理三条路径耦合较高，需防止单点改动导致行为分叉。

## 当前状态
- 截至 2026-02-23：M0、M1、M2、M3 已完成（阶段收口）。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-ENGINEERING-006 | 文档内既有任务条目 | `test_tier_required` | `./scripts/doc-governance-check.sh` + 引用可达性扫描 | 迁移文档命名一致性与可追溯性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-DOC-MIG-20260303 | 逐篇阅读后人工重写为 `.prd` 命名 | 仅批量重命名 | 保证语义保真与审计可追溯。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章 Executive Summary。
- 原“范围” -> 第 2 章 User Experience & Functionality。
- 原“接口 / 数据” -> 第 4 章 Technical Specifications。
- 原“里程碑/风险” -> 第 5 章 Risks & Roadmap。
