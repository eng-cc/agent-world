# Agent World Runtime：代码执行性能采集/统计/分析基础（2026-02-25）

## 目标
- 为 `agent_world` runtime/simulator 建立一套“代码执行性能（不卡顿）”的内建观测基础，形成采集、统计、分析闭环。
- 将采集结果作为 `RunnerMetrics` 的一部分输出，打通到现有 viewer/live/demo/长跑脚本链路。
- 在不引入外部监控基础设施的前提下，先落地可复用、低侵入、可测试的本地性能画像能力。

## 范围

### In Scope
- 在 simulator 侧新增 runtime perf 模块，支持以下阶段的 wall time 采集：
  - `tick`
  - `decision`
  - `action_execution`
  - `callback`（`on_action_result`）
- 提供窗口化统计与累计统计：
  - `avg/p50/p95/p99/min/max`
  - `samples_total/samples_window`
  - `over_budget_total/over_budget_ratio_ppm`
- 提供分析结论：
  - `health`（`unknown/healthy/warn/critical`）
  - `bottleneck`（`none/tick/decision/action_execution/callback`）
- 将快照接入：
  - `RunnerMetrics`
  - `world_llm_agent_demo --report-json`
  - `world_viewer_live` metrics 发送链路（LLM driver）
- 扩展 `scripts/llm-longrun-stress.sh` 读取并汇总 runtime perf 指标。
- 补齐单测与回归测试（`test_tier_required` / 脚本闭环）。

### Out of Scope
- Prometheus/OpenTelemetry exporter。
- 跨进程持久化时序数据库。
- 火焰图/采样 profiler 集成（pprof/tracy 等）。
- 全链路自动调参（本阶段只做观测和分析，不做自动治理）。

## 接口 / 数据

### 新增核心结构
- `RuntimePerfSeriesSnapshot`
  - 单阶段统计快照。
  - 字段：`samples_total/samples_window/last_ms/avg_ms/p50_ms/p95_ms/p99_ms/min_ms/max_ms/budget_ms/over_budget_total/over_budget_ratio_ppm`
- `RuntimePerfSnapshot`
  - 组合快照：`tick/decision/action_execution/callback` + `health` + `bottleneck` + `sample_window`
- `RuntimePerfHealth`
  - 枚举：`unknown | healthy | warn | critical`
- `RuntimePerfBottleneck`
  - 枚举：`none | tick | decision | action_execution | callback`

### 采集策略
- 数据来源：`std::time::Instant` wall time。
- 采样时机：
  - `AgentRunner::tick`：采集 `tick`，并在内部采集 `decision/action_execution/callback`。
  - `AgentRunner::tick_decide_only`：采集 `tick/decision`。
  - `AgentRunner::notify_action_result`：采集 `callback`。
  - 外部执行 action 的路径（例如共识回放、demo defer 执行）通过显式接口补录 `action_execution`。
- 窗口模型：
  - 固定窗口（默认 `512` 样本）用于分位数与短期画像。
  - 累计计数用于长期 over-budget 比例判断。

### 分析策略（MVP）
- `health` 判定：
  - `critical`：任一阶段 `p95_ms > budget_ms * 2` 或 `over_budget_ratio_ppm >= 200000`
  - `warn`：任一阶段 `p95_ms > budget_ms` 或 `over_budget_ratio_ppm >= 50000`
  - `healthy`：有样本且未触发上述阈值
  - `unknown`：无样本
- `bottleneck` 判定：
  - 取 `tick/decision/action_execution/callback` 中 `p95_ms` 最大的阶段；
  - 若全部无样本或全为 0，标记 `none`。

### 默认预算（MVP）
- `tick_budget_ms = 33`
- `decision_budget_ms = 20`
- `action_execution_budget_ms = 20`
- `callback_budget_ms = 10`

### 对外输出
- `RunnerMetrics` 增加 `runtime_perf: RuntimePerfSnapshot`（默认值可反序列化兼容）。
- `world_llm_agent_demo`：
  - `report.json` 增加 `runtime_perf` 段；
  - stdout 增加关键分析行（health/bottleneck/tick p95/over-budget）。
- `llm-longrun-stress.sh`：
  - 场景 summary 增加 runtime perf 关键字段；
  - 多场景聚合报告保留每场景 runtime perf 诊断。

## 里程碑
- `RPOF-1`：设计文档与项目管理文档落地。
- `RPOF-2`：runtime perf 模块与 `AgentRunner` 采集接线完成。
- `RPOF-3`：`RunnerMetrics`、`world_llm_agent_demo`、`world_viewer_live` 输出链路接通。
- `RPOF-4`：`llm-longrun-stress.sh` 汇总扩展 + 测试回归 + 文档/日志收口。

## 测试计划
- 单测（`test_tier_required`）：
  - runtime perf 统计正确性（percentile、over-budget、health/bottleneck）。
  - runner 路径采样覆盖（`tick`/`tick_decide_only`/`notify_action_result`）。
- 集成回归：
  - `world_llm_agent_demo` 输出含 runtime perf 字段。
  - `llm-longrun-stress.sh` 可读取 runtime perf 字段并生成 summary。
- 命令口径遵循 `testing-manual.md` 与仓库 cargo 约束（`env -u RUSTC_WRAPPER`）。

## 风险
- `Instant` wall time 受环境抖动影响，短窗口数据可能有噪声；通过窗口 + ppm 降噪。
- 性能采集本身引入额外开销；采用常量级采样与小窗口，避免重度统计。
- 新增 `RunnerMetrics` 字段可能影响历史反序列化；通过 `serde(default)` 保持兼容。
- `tick_decide_only` 场景的 action 执行在 runner 外部，若漏接补录会低估 `action_execution`；需统一在关键路径补录。 
