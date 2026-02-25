# Agent World：Viewer 性能瓶颈观测能力补齐（2026-02-25）

## 目标
- 在现有 Viewer 性能指标（FPS/帧时/over-budget）基础上，补齐“可定位瓶颈点”的观测字段。
- 让压测脚本输出不仅能判定 pass/fail，还能明确指向主要瓶颈类别（渲染帧、Runtime 阶段、标签/覆盖层容量、事件积压）。

## 范围

### In Scope
- 扩展 `RenderPerfSummary`，增加瓶颈定位所需字段：
  - Runtime 健康度与 bottleneck（来自 `RunnerMetrics.runtime_perf`）
  - Runtime 分阶段 p95（tick/decision/action_execution/callback/llm_api）
  - 客户端容量压力标记（label/overlay/event backlog）
- 新增统一瓶颈推断规则（`PerfHotspot`）：
  - `runtime_tick/runtime_decision/runtime_action_execution/runtime_callback/runtime_llm_api`
  - `render_frame/label_capacity/overlay_capacity/event_backlog/none`
- 扩展 `perf_probe` 输出，携带上述字段，供脚本自动采集。
- 扩展 `scripts/viewer-owr4-stress.sh`：
  - 解析并输出 hotspot/runtime 字段到 `metrics.csv`/`summary.json`
  - 在摘要中直观展示 bottleneck 线索。
- 补充单测与脚本回归（语法 + 最短窗 smoke）。

### Out of Scope
- 新增 GPU timestamp query、DrawCall、VRAM 真实采集。
- 新增外部时序系统（Prometheus/Otel）。
- 重写 Playwright Web 渲染性能脚本框架。

## 接口 / 数据

### RenderPerfSummary 新增字段
- `runtime_health`
- `runtime_bottleneck`
- `runtime_tick_p95_ms`
- `runtime_decision_p95_ms`
- `runtime_action_execution_p95_ms`
- `runtime_callback_p95_ms`
- `runtime_llm_api_p95_ms`
- `runtime_llm_api_budget_ms`
- `label_capacity_hit`
- `overlay_capacity_hit`
- `event_backlog_hit`

### PerfHotspot 规则（MVP）
优先级（从高到低）：
1. `runtime_llm_api`：`llm_api.p95 > llm_api.budget`
2. `runtime_*`：`runtime_health in {warn,critical}` 且 `runtime_bottleneck != none`
3. `overlay_capacity`
4. `label_capacity`
5. `event_backlog`
6. `render_frame`：`frame_p95 > 33ms` 或 `auto_degrade_active`
7. `none`

### perf_probe 输出扩展
在现有行后追加键值：
- `hotspot`
- `runtime_health`
- `runtime_bottleneck`
- `runtime_tick_p95`
- `runtime_decision_p95`
- `runtime_action_p95`
- `runtime_callback_p95`
- `runtime_llm_api_p95`
- `label_capacity_hit`
- `overlay_capacity_hit`
- `event_backlog_hit`

### viewer-owr4-stress 输出扩展
- `metrics.csv` 新增列：
  - `hotspot_primary`
  - `runtime_health_last`
  - `runtime_bottleneck_last`
  - `runtime_tick_p95_peak`
  - `runtime_decision_p95_peak`
  - `runtime_action_p95_peak`
  - `runtime_callback_p95_peak`
  - `runtime_llm_api_p95_peak`
  - `label_capacity_seen`
  - `overlay_capacity_seen`
  - `event_backlog_seen`

## 里程碑
- M1：文档建档（设计 + 项目管理）。
- M2：Viewer 指标扩展与 hotspot 推断落地。
- M3：`perf_probe` 与 `viewer-owr4-stress.sh` 接线并验证输出。
- M4：测试回归 + 文档/日志收口。

## 风险
- 规则过于激进导致 hotspot 抖动：通过固定优先级 + 最小口径先落地。
- 旧 CSV 基线兼容：新列追加到尾部，保持原字段顺序不变。
- runtime 指标在 script 场景为空：保留 `unknown/none`，不误判为 runtime 瓶颈。
