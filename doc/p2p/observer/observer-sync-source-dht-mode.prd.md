# Agent World Runtime：Observer 同步源策略化（DHT 组合链路，设计文档）

审计轮次: 3

## ROUND-002 主从口径（2026-03-05）
- 本文档在 ROUND-002 判定为 `observer-sync-source-mode` 的 DHT 增量子文档。
- 主文档入口：
  - `doc/p2p/observer/observer-sync-source-mode.prd.md`
  - `doc/p2p/observer/observer-sync-source-mode.prd.project.md`
- 本文档仅保留 DHT 组合链路差异（`HeadSyncSourceModeWithDht`）与对应实现追溯。

## 1. Executive Summary
- Problem Statement: 为 `ObserverClient` 增加 DHT 组合链路下的 head 同步源策略，显式控制“网络+DHT路径”与“路径索引路径”的切换。
- Proposed Solution: 在网络+DHT链路失败时支持回退到路径索引，提升本地恢复鲁棒性。
- Success Criteria:
  - SC-1: 保持现有 API 兼容，不影响已落地的非 DHT 策略接口。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Observer 同步源策略化（DHT 组合链路，设计文档） 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 定义 `HeadSyncSourceModeWithDht` 策略枚举。
  - AC-2: 在 `ObserverClient` 增加 DHT 组合模式接口：同步/报告/结果/跟随循环。
  - AC-3: 支持模式：
  - AC-4: `NetworkWithDhtOnly`
  - AC-5: `PathIndexOnly`
  - AC-6: `NetworkWithDhtThenPathIndex`
- Non-Goals:
  - 新增全局配置中心或动态热更新策略。
  - 指标上报系统（Prometheus/OpenTelemetry）接入。
  - 多级回退链（如 Network -> DHT -> Network(no DHT) -> PathIndex）。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/observer/observer-sync-source-dht-mode.prd.md`
  - `doc/p2p/observer/observer-sync-source-dht-mode.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 策略枚举（草案）
- `HeadSyncSourceModeWithDht::NetworkWithDhtOnly`
- `HeadSyncSourceModeWithDht::PathIndexOnly`
- `HeadSyncSourceModeWithDht::NetworkWithDhtThenPathIndex`

### 语义约束
- `NetworkWithDhtOnly`：仅走现有 `sync_from_heads_with_dht` 链路，失败直接返回。
- `PathIndexOnly`：仅走路径索引读取。
- `NetworkWithDhtThenPathIndex`：先走网络+DHT；仅在该链路报错时回退路径索引。

## 5. Risks & Roadmap
- Phased Rollout:
  - OSDM-1：设计文档与项目管理文档落地。
  - OSDM-2：实现 `HeadSyncSourceModeWithDht` 与 `ObserverClient` 模式化接口。
  - OSDM-3：补齐单元测试并完成 `agent_world_net` 回归。
  - OSDM-4：回写状态文档与 devlog。
- Technical Risks:
  - DHT 失败回退路径索引可能掩盖上游网络问题，需要保留错误上下文。
  - 模式枚举扩展后若命名不清晰，调用方容易误用，需要保持语义直观。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-109-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-109-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
