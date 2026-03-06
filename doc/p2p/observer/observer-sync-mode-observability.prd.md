# Agent World Runtime：Observer 同步源策略可观测性

审计轮次: 3

## ROUND-002 主从口径
- 当前文档降级为增量子文档，仅维护“策略可观测性”专题增量约束。
- 主入口：`doc/p2p/observer/observer-sync-mode-runtime-metrics.prd.md`；跨专题统一口径以主文档为准。

## 1. Executive Summary
- Problem Statement: 为 `ObserverClient` 的模式化同步接口补充“是否触发回退”的显式观测结果，降低故障排查成本。
- Proposed Solution: 保持现有接口行为不变，在不破坏兼容性的前提下新增可观测报告接口。
- Success Criteria:
  - SC-1: 同时覆盖非 DHT 与 DHT 组合模式。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Observer 同步源策略可观测性 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 为 `HeadSyncSourceMode` 增加可观测报告接口，输出：
  - AC-2: `mode`
  - AC-3: `drained/applied`（沿用 `HeadSyncReport`）
  - AC-4: `fallback_used`
  - AC-5: 为 `HeadSyncSourceModeWithDht` 增加同类可观测报告接口。
  - AC-6: 在内部模式分发流程中记录是否发生回退。
- Non-Goals:
  - 引入全局 metrics 系统（Prometheus/OpenTelemetry）。
  - 持久化回退统计数据。
  - 对现有 `HeadSyncReport` 结构做破坏性字段变更。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/observer/observer-sync-mode-observability.prd.md`
  - `doc/p2p/observer/observer-sync-mode-observability.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 新增报告结构（草案）
- `HeadSyncModeReport`：
  - `mode: HeadSyncSourceMode`
  - `report: HeadSyncReport`
  - `fallback_used: bool`
- `HeadSyncModeWithDhtReport`：
  - `mode: HeadSyncSourceModeWithDht`
  - `report: HeadSyncReport`
  - `fallback_used: bool`

### 新增接口（草案）
- `sync_heads_with_mode_observed_report`
- `sync_heads_with_dht_mode_observed_report`

## 5. Risks & Roadmap
- Phased Rollout:
  - OSMO-1：设计文档与项目管理文档落地。
  - OSMO-2：实现可观测报告结构与接口。
  - OSMO-3：补齐测试并完成 `agent_world_net` 回归。
  - OSMO-4：回写状态文档与 devlog。
- Technical Risks:
  - 若回退标识计算位置不一致，可能与实际执行路径偏离，需要统一在模式分发层计算。
  - 新增报告接口若与旧接口语义不清，可能导致调用方误用，需要保持命名直观。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-107-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-107-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
