# Agent World Runtime：Membership Recovery 调度门控与计数聚合算术语义硬化（15 点清单第十二阶段）

审计轮次: 3

- 对应项目管理文档: doc/world-runtime/runtime/runtime-numeric-correctness-phase12.prd.project.md

## 1. Executive Summary
- 收口 `membership_recovery/mod.rs` 中仍使用饱和算术的调度门控与关键计数聚合路径。
- 在极端时间边界与计数边界下，将“静默饱和继续执行”升级为“显式失败且不污染状态”。
- 保持 recovery/dead-letter 写入原子性：失败路径不更新 `last_replay_at_ms`、不写入部分 pending/dead-letter 结果。

## 2. User Experience & Functionality
### In Scope（第十二阶段）
- `crates/agent_world_consensus/src/membership_recovery/mod.rs`
  - `run_revocation_dead_letter_replay_schedule` 的 interval gate 时间差从 `saturating_sub` 改为受检减法。
  - `emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter` 的关键容量/计数累加从饱和语义改为受检语义（越界显式失败）。
  - 统一溢出错误语义为 `WorldError::DistributedValidationFailed`，错误消息包含关键现场值。
- 测试
  - 在 `membership_recovery_tests.rs` 增加时间差溢出拒绝测试与计数边界拒绝测试。
  - 验证失败路径不写 `last_replay_at_ms`，不提交部分 recovery/dead-letter 状态。

### Out of Scope（后续阶段）
- `membership.rs`、`mempool.rs` 等非本阶段目标文件的全量计数语义统一。
- 全链路数值 newtype 与跨模块统一时钟治理。


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- 对外 API 入口保持不变（继续返回 `Result<..., WorldError>`）。
- 内部将新增/复用受检加法 helper，替换关键 `saturating_add`。
- 失败模型统一：
  - `WorldError::DistributedValidationFailed`
  - 错误消息包含 `now_ms/last_replay_at_ms` 或计数字段上下文。

## 5. Risks & Roadmap
- M0：Phase12 建档并冻结边界。
- M1：replay schedule 时间门控受检语义改造完成。
- M2：recovery 计数/容量受检语义改造与边界测试完成。
- M3：回归测试通过并完成文档/devlog 收口。

### Technical Risks
- 从饱和语义转为显式失败后，历史“边界夹逼继续执行”路径会变为拒绝，需要同步更新测试预期。
- 计数路径分支较多，若改造不一致，可能导致 report/metrics 与落盘状态不一致。
- 需重点验证失败发生在状态写入前，避免 pending/dead-letter 半更新。

## 当前状态
- 截至 2026-02-23：M0、M1、M2、M3 已完成（Phase12 收口完成）。

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
