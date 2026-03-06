# Agent World Runtime：Membership Replay 调度/冷却时间门控算术语义硬化（15 点清单第十一阶段）

审计轮次: 4

- 对应项目管理文档: doc/world-runtime/runtime/runtime-numeric-correctness-phase11.prd.project.md

## 1. Executive Summary
- 收口 `membership_recovery/replay.rs` 中调度间隔、策略冷却、rollback 冷却三条时间门控的饱和减法语义。
- 在极端时间边界（如 `i64::MIN` 历史时间戳）下，从“静默饱和并继续执行”切换为“显式失败并阻断状态写入”。
- 保持回放状态与策略状态一致性：失败路径不更新 `last_replay_at_ms`、不覆盖 policy store。

## 2. User Experience & Functionality
### In Scope（第十一阶段）
- `crates/agent_world_consensus/src/membership_recovery/replay.rs`
  - `run_revocation_dead_letter_replay_schedule_with_state_store` 的 interval gate 时间差改为受检减法。
  - `recommend_revocation_dead_letter_replay_policy_with_adaptive_guard` 的 policy cooldown 时间差改为受检减法。
  - `should_rollback_to_stable_policy` 的 rollback cooldown 时间差改为受检减法（并沿调用链显式失败）。
- 测试：
  - 在 `membership_dead_letter_replay_tests.rs` 与 `membership_dead_letter_replay_persistence_tests.rs` 新增时间差溢出拒绝测试。
  - 验证 overflow 时不写 replay state / policy state。

### Out of Scope（后续阶段）
- replay 模块中全部 `usize` 聚合计数去饱和化（本阶段仅处理高风险时间门控）。
- membership_reconciliation / mempool 等其他子模块算术收口。
- 全链路时间类型 newtype 与时钟统一治理。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- 保持公开 API 入口不变（仍为 `Result<..., WorldError>`）。
- 内部 helper 语义升级：
  - `should_rollback_to_stable_policy` 从布尔返回升级为可失败返回，调用方透传错误。
- 统一错误模型：
  - `WorldError::DistributedValidationFailed`，错误消息包含 `now_ms/last_*_ms` 现场值。

## 5. Risks & Roadmap
- M0：Phase11 建档并冻结边界。
- M1：调度间隔与 policy cooldown 受检语义改造完成。
- M2：rollback cooldown 受检语义改造与边界测试完成。
- M3：回归测试通过并完成文档/devlog 收口。

### Technical Risks
- 从饱和语义转向显式失败后，历史“时间回拨容忍”路径会改变，需要同步更新测试预期。
- helper 返回类型变化会影响持久化策略调用链，若接线遗漏可能导致行为分裂。
- 需重点验证失败发生在状态写入前，避免 replay/policy 半更新。

## 当前状态
- 截至 2026-02-23：M0~M3 全部完成（调度间隔/policy cooldown/rollback cooldown 三条时间门控均已受检化并完成回归）。

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
