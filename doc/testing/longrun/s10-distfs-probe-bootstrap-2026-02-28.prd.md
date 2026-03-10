# Agent World：S10 DistFS Probe Bootstrap（2026-02-28）

- 对应设计文档: `doc/testing/longrun/s10-distfs-probe-bootstrap-2026-02-28.design.md`
- 对应项目管理文档: `doc/testing/longrun/s10-distfs-probe-bootstrap-2026-02-28.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: S10 长跑中 reward runtime 的 DistFS 统计可能长期为 `distfs_total_checks=0`，导致 `metric_gate=insufficient_data`，使发布门禁无法得出有效结论。
- Proposed Solution: 在 reward worker 启动阶段增加最小 probe seed bootstrap，仅当 `storage_root/blobs` 为空时写入幂等 seed blob，确保 DistFS 在长窗内产生可统计样本。
- Success Criteria:
  - SC-1: 当 blob 集为空时自动写入 seed，且重复启动不产生无界重复写入。
  - SC-2: S10 运行中 `status.reward_runtime.distfs_total_checks` 可增长到 `>0`。
  - SC-3: 在其他指标正常时，`summary.json.run.metric_gate.status` 可从 `insufficient_data` 恢复为 `pass`。
  - SC-4: 不修改 DistFS 算法阈值语义，不降低现有门禁强度。
  - SC-5: 手册与项目文档可追溯到统一验收口径。

## 2. User Experience & Functionality
- User Personas:
  - 测试维护者：希望 S10 门禁能基于真实 DistFS 样本做判定。
  - 发布负责人：希望避免“数据不足”导致的假阻塞或不可判定。
  - Runtime 维护者：希望修复方案最小侵入且幂等。
- User Scenarios & Frequency:
  - 每次 S10 基线长跑都会触发启动 bootstrap 检查。
  - 发布前验证场景需要审查 DistFS 样本是否进入统计窗口。
  - 问题复盘时按 summary/status 字段定位是否为样本缺失。
- User Stories:
  - PRD-TESTING-LONGRUN-S10DISTFS-001: As a 测试维护者, I want runtime to auto-bootstrap DistFS probe seed when storage is empty, so that metric gate has valid data.
  - PRD-TESTING-LONGRUN-S10DISTFS-002: As a 发布负责人, I want S10 metric gate to reflect true runtime health instead of insufficient data, so that release decisions are reliable.
  - PRD-TESTING-LONGRUN-S10DISTFS-003: As a runtime 维护者, I want the bootstrap flow to be idempotent and non-blocking, so that startup reliability is preserved.
- Critical User Flows:
  1. Flow-S10DISTFS-001: `reward worker 启动 -> 检查 storage_root/blobs 是否为空 -> 决定是否写入 seed blob`
  2. Flow-S10DISTFS-002: `长跑持续运行 -> DistFS probe 产生检查样本 -> status/summary 聚合`
  3. Flow-S10DISTFS-003: `发布验收 -> 审阅 metric_gate 与 distfs_total_checks -> 结论入档`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 启动 bootstrap | `storage_root/blobs`、seed 内容、写入结果 | 启动时执行空集检测并条件写入 | `init -> checked -> seeded/skipped` | 仅空集写入；非空跳过 | runtime 自动执行 |
| DistFS 指标采样 | `distfs_total_checks`、`metric_gate.status` | 运行中持续采样与聚合 | `insufficient_data -> pass/fail` | 以长窗累计样本判定 | 测试/发布可读 |
| 异常记录 | `last_error`、日志条目 | 写入失败记录错误但不中断主流程 | `error_recorded` | 错误优先保留原始信息 | 维护者可审阅 |
- Acceptance Criteria:
  - AC-1: 代码中实现“仅空 blob 集写入 seed”的幂等逻辑。
  - AC-2: 失败场景记录 `last_error`，不阻断 worker 主循环。
  - AC-3: S10 复跑后 DistFS 相关状态字段可观测、可追踪。
  - AC-4: 手册与项目管理文档同步更新并保持一致。
  - AC-5: 文档迁移采用 strict schema，包含原文约束点映射。
- Non-Goals:
  - 不更改 DistFS probe 算法与阈值。
  - 不更改 S9/S10 的失败策略与 gate 语义。
  - 不引入新的外部依赖或额外控制面接口。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为 runtime 启动与指标可用性修复，不涉及 AI 推理或模型编排）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 在 `reward_runtime_worker.rs` 启动路径增加 DistFS probe seed bootstrap，作为预热步骤向本地存储注入最小样本，之后由现有 DistFS 检查链路按既有语义统计。
- Integration Points:
  - `crates/agent_world/src/bin/world_chain_runtime/reward_runtime_worker.rs`
  - `scripts/s10-five-node-game-soak.sh`
  - `testing-manual.md`
  - `doc/testing/longrun/s10-distfs-probe-bootstrap-2026-02-28.project.md`
- Edge Cases & Error Handling:
  - 存储目录不可写：记录 `last_error` 并继续主流程，等待真实业务样本补齐。
  - 重启频繁：通过“仅空集写入”保障幂等，避免重复污染。
  - 指标短窗仍无样本：保留 `insufficient_data` 并通过状态字段显式暴露。
  - 路径异常或权限不足：输出明确错误签名，便于回归定位。
- Non-Functional Requirements:
  - NFR-S10DISTFS-1: 启动 bootstrap 不显著增加启动耗时（相对基线波动 <= 5%）。
  - NFR-S10DISTFS-2: seed 写入逻辑幂等，重复运行不引入无界增长。
  - NFR-S10DISTFS-3: 指标字段具备可观测性，发布审核可在 30 分钟内完成追溯。
  - NFR-S10DISTFS-4: 文档与手册口径一致，引用路径统一为 `.prd.md/.project.md`。
- Security & Privacy: seed 内容使用最小化非敏感数据；日志不暴露敏感凭据。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (S10DISTFS-1): 设计与项目文档建档。
  - v1.1 (S10DISTFS-2): 实现启动 bootstrap 逻辑。
  - v2.0 (S10DISTFS-3): 复跑 S10 基线并验证 status/summary 口径。
  - v2.1 (S10DISTFS-4): 手册与任务文档收口。
  - v2.2 (S10DISTFS-5): 专题文档迁移到 strict schema 并统一 `.prd` 命名。
- Technical Risks:
  - 风险-1: 启动写入失败仍可能造成短时 `insufficient_data`。
  - 风险-2: seed 策略设计不当会影响样本代表性解释。
  - 风险-3: 文档路径迁移遗漏会造成引用断链。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-LONGRUN-S10DISTFS-001 | S10DISTFS-1/2 | `test_tier_required` | 启动路径与空集条件写入逻辑检查 | reward worker 启动阶段 |
| PRD-TESTING-LONGRUN-S10DISTFS-002 | S10DISTFS-2/3/4 | `test_tier_required` | S10 基线复跑并核验 status/summary 字段 | 发布 gate 指标可判定性 |
| PRD-TESTING-LONGRUN-S10DISTFS-003 | S10DISTFS-3/4/5 | `test_tier_required` | 异常注入与文档治理检查 | 幂等性、追溯与维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-S10DISTFS-001 | 启动阶段空集时自动写入最小 seed | 降低 gate 阈值绕过样本不足 | 保持门禁强度且补齐真实统计前提。 |
| DEC-S10DISTFS-002 | 写入失败仅记录错误不阻断主流程 | 写入失败即终止 runtime | 长跑稳定性优先，避免单点写入导致全局中断。 |
| DEC-S10DISTFS-003 | 文档迁移采用人工重写并保留约束映射 | 批量脚本转换 | 避免语义丢失并提升审阅可用性。 |

## 原文约束点映射（内容保真）
- 原“目标：修复 `distfs_total_checks=0` 与 `metric_gate=insufficient_data`” -> 第 1 章 Problem/Solution/SC。
- 原“范围：reward runtime 代码 + S10 复跑 + 手册/devlog 收口” -> 第 2 章 AC 与第 4 章 Integration Points。
- 原“非目标：不改算法阈值、不改 S9/S10 失败策略” -> 第 2 章 Non-Goals。
- 原“接口/数据：启动空集检测并 seed，状态字段变化” -> 第 2 章规格矩阵 + 第 4 章架构与边界异常。
- 原“里程碑 M1~M4” -> 第 5 章 phased rollout（S10DISTFS-1~5）。
- 原“风险：重复写入、写入失败噪声” -> 第 4 章 Edge Cases + 第 5 章 Technical Risks。
