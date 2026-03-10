# Agent World Runtime：Governance Drill/Retention 时间算术数值语义硬化（15 点清单第九阶段）

- 对应设计文档: `doc/world-runtime/runtime/runtime-numeric-correctness-phase9.design.md`
- 对应项目管理文档: `doc/world-runtime/runtime/runtime-numeric-correctness-phase9.project.md`

审计轮次: 4


## 1. Executive Summary
- 收口 `membership_recovery/replay_archive` 中治理审计保留与恢复演练调度的时间算术饱和语义。
- 将 `scheduled_at_ms +/- interval` 与 `now_ms - audited_at_ms` 从“静默饱和”改为“显式失败”，避免极端时间边界下误调度或误保留。
- 保持失败不污染状态：溢出时不写入 schedule state，不覆盖 audit retention 存储。

## 2. User Experience & Functionality
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


## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（文档迁移任务）。
- Evaluation Strategy: 通过文档治理校验、引用扫描与任务日志检查验证迁移质量。

## 4. Technical Specifications
- 不改变公开接口签名（均保持 `Result<..., WorldError>`）。
- 仅调整内部时间算术实现：
  - 溢出时统一返回 `WorldError::DistributedValidationFailed`，包含字段上下文。

## 5. Risks & Roadmap
- M0：Phase9 建档并冻结范围。
- M1：drill schedule 时间算术受检语义落地。
- M2：audit retention 时间算术受检语义落地与测试补齐。
- M3：回归测试通过并完成文档/devlog 收口。

### Technical Risks
- 由“饱和继续执行”切换为“显式失败”后，极端边界下行为预期会变化。
- 若只改部分路径，可能导致调度链条语义分裂，需要在单阶段内收口。
- 需确保错误返回发生在写入之前，避免半更新状态。

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
