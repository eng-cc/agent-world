# oasis7 Runtime：LLM API 延迟与代码执行耗时解耦（2026-02-25）

- 对应设计文档: `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.design.md`
- 对应项目管理文档: `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.project.md`

审计轮次: 4

## 1. Executive Summary
- Problem Statement: runtime perf 的 `decision` 指标混入 LLM 云端 API 网络时延后，会掩盖本地代码执行真实性能，导致“代码卡顿”判断被外部依赖噪声污染。
- Proposed Solution: 将 `llm_api` 作为独立 runtime perf 阶段，并把 `decision` 重新定义为本地执行耗时（`decision_local_ms`），保持 health/bottleneck 仅基于本地四阶段。
- Success Criteria:
  - SC-1: `RuntimePerfSnapshot` 新增 `llm_api` 序列并可完整输出统计。
  - SC-2: `decision` 统计口径改为 `max(total_decision_ms - llm_api_ms, 0)`。
  - SC-3: `health`/`bottleneck` 判定不受 `llm_api` 延迟波动影响。
  - SC-4: runner 单测覆盖拆分与边界（负值、缺失、非有限数值）处理。
  - SC-5: 现有 `RunnerMetrics` / demo / JSON 报告链路兼容新增字段。

## 2. User Experience & Functionality
- User Personas:
  - runtime 维护者：需要区分本地执行慢与云端 API 慢。
  - 测试维护者：需要稳定判断代码回归，不被网络抖动误报。
  - 发布负责人：需要可解释的性能证据，清晰区分内部/外部瓶颈。
- User Scenarios & Frequency:
  - 每次 LLM 路径变更后，检查 `decision` 与 `llm_api` 是否正确拆分。
  - 高 action 负载回归时，比较本地阶段瓶颈与 API 延迟波动。
  - 发布评审时，以 `health/bottleneck` 判定本地风险，以 `llm_api` 作为旁路诊断。
- User Stories:
  - PRD-TESTING-PERF-RPOFLLM-001: As a runtime 维护者, I want decision time to exclude LLM API latency, so that local execution regressions are measurable.
  - PRD-TESTING-PERF-RPOFLLM-002: As a 测试维护者, I want llm_api latency tracked separately, so that network slowness is diagnosable without polluting runtime health.
  - PRD-TESTING-PERF-RPOFLLM-003: As a 发布负责人, I want health/bottleneck to stay code-local while still exposing llm_api diagnostics, so that release risk decisions are consistent.
- Critical User Flows:
  1. Flow-RPOFLLM-001: `runner 完成一次 decision -> 读取 trace.latency_ms -> 计算 decision_local + llm_api`
  2. Flow-RPOFLLM-002: `更新 runtime perf 序列 -> 生成 snapshot -> 输出 RunnerMetrics`
  3. Flow-RPOFLLM-003: `demo/report 消费 runtime_perf -> 分别展示本地执行与 llm_api 诊断`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 指标拆分 | `decision_local_ms`、`llm_api_ms` | 每次 decision 后计算并入库 | `raw -> split` | `decision_local=max(total-llm,0)` | runtime 自动执行 |
| 新增序列 | `RuntimePerfSnapshot.llm_api` | 采样后更新 `avg/p95/over_budget` 等字段 | `empty -> sampled` | 使用独立预算 `llm_api_budget_ms=1000` | 测试/发布可读 |
| 结论口径 | `health`、`bottleneck` | 仅基于 `tick/decision/action_execution/callback` 判定 | `unknown -> healthy/warn/critical` | `llm_api` 不参与排名与健康阈值 | 维护者审阅 |
| 边界钳制 | 负值/非有限数值处理 | 异常值归零并避免污染统计 | `sanitized` | 无效值记 0 | 系统自动 |
- Acceptance Criteria:
  - AC-1: runtime perf 含 `llm_api` 字段且链路序列化兼容。
  - AC-2: runner 采样改为 decision/llm_api 双轨，公式符合定义。
  - AC-3: health/bottleneck 单测确认 `llm_api` 不影响结论。
  - AC-4: 高负载 + 真实 LLM 回归可观测“本地慢”和“API 慢”的分离信号。
  - AC-5: 文档迁移完成 strict schema 与 `.prd` 命名统一。
- Non-Goals:
  - 不修改 LLM 客户端重试/超时/限流策略。
  - 不引入外部监控平台或新业务语义改动。
  - 不改变场景配置策略，仅调整观测口径。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用（本专题为 runtime perf 口径治理，不涉及 AI 模型能力改造）。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 在既有 runtime perf 基础上新增 `llm_api` 序列，并在 runner 采样阶段将总 decision 时长拆分为本地执行与 API 延迟两部分；健康结论继续由本地四阶段决定，`llm_api` 仅作为旁路诊断信号。
- Integration Points:
  - `crates/oasis7/src/simulator/runtime_perf.rs`
  - `crates/oasis7/src/simulator/runner.rs`
  - `crates/oasis7/src/simulator/tests/runner.rs`
  - `doc/testing/performance/runtime-performance-observability-foundation-2026-02-25.prd.md`
  - `testing-manual.md`
  - `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.project.md`
- Edge Cases & Error Handling:
  - `llm_diagnostics.latency_ms` 缺失：`llm_api` 可能低估，但不影响本地结论稳定性。
  - `llm_api_ms >= total_decision_ms`：`decision_local` 钳制为 `0`。
  - 非有限数值与负值：统一按 `0` 处理，防止异常放大。
  - 下游解析旧结构：通过 `serde(default)` 保持字段兼容。
- Non-Functional Requirements:
  - NFR-RPOFLLM-1: 口径拆分逻辑可重复、可测试、无歧义。
  - NFR-RPOFLLM-2: 新增字段不破坏既有 report/json 消费路径。
  - NFR-RPOFLLM-3: 诊断信息可在 required 回归中快速区分本地与外部瓶颈。
  - NFR-RPOFLLM-4: 采样额外开销保持低侵入。
- Security & Privacy: 指标仅含延迟与统计字段，不暴露敏感输入输出内容。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (RPOF-L1): 设计与项目文档建档。
  - v1.1 (RPOF-L2): runtime perf 拆分实现与输出接线。
  - v2.0 (RPOF-L3): 单测与高负载回归验证。
  - v2.1 (RPOF-L4): 文档/日志收口。
  - v2.2 (RPOF-L5): strict schema 迁移与命名统一。
- Technical Risks:
  - 风险-1: trace 缺失导致 `llm_api` 低估。
  - 风险-2: 减法误差与异常值污染统计。
  - 风险-3: 下游脚本未及时兼容新增字段。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-TESTING-PERF-RPOFLLM-001 | RPOF-L1/L2/L3 | `test_tier_required` | runner decision/llm_api 拆分单测与边界断言 | runtime 本地执行性能口径 |
| PRD-TESTING-PERF-RPOFLLM-002 | RPOF-L2/L3/L4 | `test_tier_required` | runtime_perf `llm_api` 序列统计与输出链路核验 | 外部 API 延迟诊断能力 |
| PRD-TESTING-PERF-RPOFLLM-003 | RPOF-L2/L4/L5 | `test_tier_required` | health/bottleneck 口径回归 + 文档治理检查 | 发布性能证据一致性 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-RPOFLLM-001 | `decision` 改为本地执行时长，`llm_api` 独立序列 | 继续混合统计 | 消除网络噪声对代码性能判定的污染。 |
| DEC-RPOFLLM-002 | `health/bottleneck` 仅看本地四阶段 | 将 `llm_api` 纳入健康判定 | 保持“代码是否卡顿”语义稳定。 |
| DEC-RPOFLLM-003 | 异常值统一钳制为 0 | 直接透传异常值 | 降低统计失真与误告警风险。 |
| DEC-RPOFLLM-004 | 通过兼容默认值扩展字段 | 破坏式改结构 | 避免下游消费中断。 |

## 原文约束点映射（内容保真）
- 原“目标：LLM API 延迟与代码执行耗时解耦、输出链路兼容” -> 第 1 章 Problem/Solution/SC。
- 原“In/Out of Scope（新增 llm_api、decision_local 公式、不改客户端策略）” -> 第 2 章规格矩阵与 Non-Goals。
- 原“接口增量、预算、采样口径、结论口径” -> 第 2 章规格矩阵 + 第 4 章技术规格。
- 原“钳制规则与边界处理” -> 第 4 章 Edge Cases & Error Handling。
- 原“里程碑 RPOF-L1~L4、测试计划” -> 第 5 章 phased rollout + 第 6 章验证追踪。
- 原“风险（trace 缺失、减法误差、脚本兼容）” -> 第 4 章边界与第 5 章风险。
