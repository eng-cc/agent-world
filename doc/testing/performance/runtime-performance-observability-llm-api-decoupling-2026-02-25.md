# Agent World Runtime：LLM API 延迟与代码执行耗时解耦（2026-02-25）

## 目标
- 将 LLM 云端 API 调用耗时从 runtime 代码执行性能统计中剥离，避免网络时延污染“代码是否卡顿”的判断。
- 保留 LLM API 延迟观测能力，作为独立阶段用于诊断外部依赖延迟。
- 保持现有 `runtime_perf` 输出链路兼容（`RunnerMetrics`、`world_llm_agent_demo`、JSON 报告）。

## 范围

### In Scope
- `runtime_perf` 新增 `llm_api` 统计阶段（独立预算、分位数、over-budget 计数）。
- `AgentRunner::tick` / `tick_decide_only` 的 `decision` 计时改为“本地代码执行耗时”：
  - `decision_local_ms = max(total_decision_ms - llm_api_ms, 0)`
  - `llm_api_ms` 来源于 `AgentDecisionTrace.llm_diagnostics.latency_ms`。
- `health` 与 `bottleneck` 继续仅基于本地代码执行阶段（`tick/decision/action_execution/callback`），不纳入 `llm_api`。
- 补充单元测试与高 action 负载回归验证。

### Out of Scope
- 修改 LLM 客户端重试策略、超时策略或限流策略。
- 引入外部监控系统（Prometheus/OpenTelemetry）。
- 改动业务动作语义或场景配置。

## 接口 / 数据

### RuntimePerfSnapshot 增量
- 新增字段：`llm_api: RuntimePerfSeriesSnapshot`。
- 预算默认值：`llm_api_budget_ms = 1000`（仅用于单阶段观测，不参与 runtime 健康结论）。

### 采样口径
- `decision`：仅记录本地代码执行耗时（排除 `llm_api`）。
- `llm_api`：仅记录 LLM 云端 API 调用往返耗时聚合值。
- 钳制规则：
  - 当 `llm_api_ms >= total_decision_ms` 时，`decision_local_ms` 记为 `0`。
  - 对非有限数值与负值按 `0` 处理，避免污染统计。

### 结论口径
- `health`：只看 `tick/decision/action_execution/callback`。
- `bottleneck`：只在上述四个本地阶段中挑选 `p95` 最大项。
- `llm_api` 只做旁路诊断，不改变 runtime 代码性能健康级别。

## 里程碑
- `RPOF-L1`：设计文档与项目管理文档建档。
- `RPOF-L2`：`runtime_perf` + `runner` 解耦实现完成。
- `RPOF-L3`：单测与高 action 负载回归完成。
- `RPOF-L4`：文档/日志回写与收口。

## 测试计划
- `test_tier_required`：
  - `runtime_perf` 新增 `llm_api` 序列统计与健康判定边界测试。
  - `runner` 新增 decision/llm_api 拆分采样测试。
- 回归验证：
  - 真实 LLM + 高 action 场景运行 `world_llm_agent_demo`，确认：
    - `runtime_perf.decision` 不再被云端延迟拉高；
    - `runtime_perf.llm_api` 能反映云端调用耗时。

## 风险
- `llm_diagnostics.latency_ms` 依赖 trace 完整性；若缺失，`llm_api` 可能低估。
- wall-time 减法存在微小误差；通过钳制规则避免负值和异常放大。
- 新增字段可能影响下游解析脚本；通过 `serde(default)` 与脚本兼容读取规避。
