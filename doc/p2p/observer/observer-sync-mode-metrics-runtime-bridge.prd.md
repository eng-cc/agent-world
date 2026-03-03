# Agent World Runtime：Observer 同步源统计桥接

## 1. Executive Summary
- Problem Statement: 将 `ObserverRuntimeMetrics` 从“可手动记录”推进到“跟随同步流程自动记录”，降低上层 runtime 接入复杂度。
- Proposed Solution: 为非 DHT 与 DHT 组合模式提供一致的“同步 + 计数”入口。
- Success Criteria:
  - SC-1: 保持现有同步 API 兼容，新增桥接接口而非破坏性改动。

## 2. User Experience & Functionality
- User Personas: 协议维护者、任务执行者、质量复核者。
- User Scenarios & Frequency: 每次专题改动前后执行需求核对、测试回归与状态回写。
- User Stories: As a 维护者, I want Agent World Runtime：Observer 同步源统计桥接 的需求结构化, so that implementation is auditable.
- Critical User Flows: `阅读旧文档 -> 重写为 strict PRD -> 回写项目文档 -> 校验提交`。
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 专题迁移 | 需求/任务/依赖/状态/测试层级 | 逐篇重写并校验 | `draft -> active -> done` | 以原文约束点映射为主线 | 维护者写入，复核者抽检 |
- Acceptance Criteria:
  - AC-1: 在 `ObserverClient` 增加“执行同步并自动写入 metrics”的桥接接口。
  - AC-2: 支持单轮同步桥接与多轮 follow 桥接。
  - AC-3: follow 桥接沿用既有 `max_rounds` 终止语义，内部按轮记录 metrics。
  - AC-4: 补齐单元测试覆盖：
  - AC-5: 回退路径下 fallback 计数自动增长。
  - AC-6: follow 场景下多轮计数与 `HeadFollowReport` 行为一致。
- Non-Goals:
  - viewer 面板 UI 直接渲染与展示。
  - 指标持久化、导出（Prometheus/OTel）。
  - 全局调度器级别的采样频率管理。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 保持原文技术边界，按 strict PRD 结构重排。
- Integration Points:
  - `doc/p2p/observer/observer-sync-mode-metrics-runtime-bridge.prd.md`
  - `doc/p2p/observer/observer-sync-mode-metrics-runtime-bridge.prd.project.md`
  - `testing-manual.md`
- Edge Cases & Error Handling: 命名不一致、章节缺失、引用断链需在同提交修复。
- Non-Functional Requirements: 非 archive 文档 <=500 行；PRD-ID/任务映射完整；治理检查通过。
- Security & Privacy: 不引入敏感信息与本地绝对路径。

### 原文技术约束（保真）
#### 接口 / 数据
### 新增桥接接口（草案）
- `sync_heads_with_mode_observed_report_and_record`
- `sync_heads_with_dht_mode_observed_report_and_record`
- `follow_heads_with_mode_and_metrics`
- `follow_heads_with_dht_mode_and_metrics`

### 语义约束
- 每次桥接接口成功产出一轮报告，必定调用对应 `record_*`。
- `follow_*` 桥接内部按轮记录，最终返回原有 `HeadFollowReport`，不改变报告聚合规则。
- 统计结构仍由调用方持有（`&mut ObserverRuntimeMetrics`），避免隐式全局状态。

## 5. Risks & Roadmap
- Phased Rollout:
  - OSMB-1：设计文档与项目管理文档落地。
  - OSMB-2：实现桥接接口与导出。
  - OSMB-3：补齐桥接接口测试并完成 `agent_world_net` 回归。
  - OSMB-4：回写状态文档与 devlog 收口。
- Technical Risks:
  - 桥接接口命名过长或语义不清，可能造成调用方误选；需保持“observed + record/metrics”可辨识。
  - follow 场景若记录时机与原有轮次判定不一致，可能导致统计偏差；需复用现有 `follow_head_sync` 语义。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-MIG-106-001 | T0~Tn | `test_tier_required` | 文档治理检查 + 章节完整性核验 | 专题文档可维护性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-PRD-P2P-MIG-106-001 | 逐篇阅读后人工重写 | 直接重命名 | 保证语义保真和可审计性。 |

## 原文约束点映射（内容保真）
- 原“目标” -> 第 1 章。
- 原“范围” -> 第 2 章。
- 原“接口/数据、里程碑、风险” -> 第 4~6 章。
