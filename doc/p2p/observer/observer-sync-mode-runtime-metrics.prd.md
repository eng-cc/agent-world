# Agent World Runtime：Observer 同步源运行态统计

- 对应设计文档: `doc/p2p/observer/observer-sync-mode-runtime-metrics.design.md`
- 对应项目管理文档: `doc/p2p/observer/observer-sync-mode-runtime-metrics.project.md`

审计轮次: 5
## ROUND-002 主从口径
- 主入口：`doc/p2p/observer/observer-sync-mode-runtime-metrics.prd.md`。
- `observer-sync-mode-metrics-runtime-bridge` 与 `observer-sync-mode-observability` 作为增量子文档维护，跨专题统一口径以本主文档为准。

## 1. Executive Summary
- Problem Statement: 将 `ObserverClient` 已有的模式化可观测报告接入运行态统计，形成“持续监控”基础闭环。
- Proposed Solution: 统一沉淀非 DHT 与 DHT 组合模式下的核心计数：`total`、`applied`、`fallback`。
- Success Criteria:
  - SC-1: 保持现有同步 API 不变，以新增结构与接口方式提供统计能力。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Observer 同步源运行态统计 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在 `agent_world_net` 新增 observer 运行态统计模块（内存计数）。
  - AC-2: 提供针对 `HeadSyncModeReport` 与 `HeadSyncModeWithDhtReport` 的记录接口。
  - AC-3: 提供快照读取接口，便于上层 runtime/面板周期拉取并展示。
  - AC-4: 补充单元测试，覆盖各模式计数正确性与回退计数。
- Non-Goals:
  - Prometheus/OpenTelemetry exporter。
  - 跨进程或落盘持久化统计。
  - 告警规则引擎与阈值策略。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/observer/observer-sync-mode-runtime-metrics.prd.md`
  - `doc/p2p/observer/observer-sync-mode-runtime-metrics.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 计数维度
- `total`: 收到一次模式化同步报告即 +1。
- `applied`: 报告 `report.applied.is_some()` 时 +1。
- `fallback`: 报告 `fallback_used == true` 时 +1。

### 新增结构（草案）
- `ObserverModeCounters`
  - `total: u64`
  - `applied: u64`
  - `fallback: u64`
- `ObserverModeRuntimeMetricsSnapshot`
  - `network_only: ObserverModeCounters`
  - `path_index_only: ObserverModeCounters`
  - `network_then_path_index: ObserverModeCounters`
- `ObserverModeWithDhtRuntimeMetricsSnapshot`
  - `network_with_dht_only: ObserverModeCounters`
  - `path_index_only: ObserverModeCounters`
  - `network_with_dht_then_path_index: ObserverModeCounters`
- `ObserverRuntimeMetricsSnapshot`
  - `mode: ObserverModeRuntimeMetricsSnapshot`
  - `mode_with_dht: ObserverModeWithDhtRuntimeMetricsSnapshot`

### 新增接口（草案）
- `ObserverRuntimeMetrics::record_mode_report(&HeadSyncModeReport)`
- `ObserverRuntimeMetrics::record_mode_with_dht_report(&HeadSyncModeWithDhtReport)`
- `ObserverRuntimeMetrics::snapshot() -> ObserverRuntimeMetricsSnapshot`

## 5. Risks & Roadmap
- Phased Rollout:
  - OSRM-1：设计文档与项目管理文档落地。
  - OSRM-2：实现运行态统计结构与导出接口。
  - OSRM-3：补齐单元测试并完成 `agent_world_net` 回归。
  - OSRM-4：回写状态文档与 devlog 收口。
- Technical Risks:
  - 统计语义若与调用方预期不一致（例如 `total` 是否按轮次或按 head 条目），会导致面板误判；需在文档中固定“按报告次数计数”。
  - 若后续扩展更多模式，存在字段膨胀风险；需要保持结构可扩展并保持向后兼容。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-108-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-108-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
