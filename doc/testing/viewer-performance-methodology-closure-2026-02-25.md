# Agent World：Viewer 性能测试方法论能力补齐（2026-02-25）

## 目标
- 将 Viewer 性能测试从“手工观察为主”升级为“可自动采集、可门禁判定、可跨版本对比”的可执行流程。
- 对齐现有工程已落地指标（`frame_ms_avg/p95`、`over_budget_pct`、`event_window_size`、`auto_degrade`），避免引入当前代码尚未提供的数据口径。
- 在不改变 Viewer 主业务链路的前提下，优先通过压测脚本补齐性能回归方法论闭环。

## 范围

### In Scope
- 增强 `scripts/viewer-owr4-stress.sh`：
  - 自动开启并采集 `perf_probe` 指标。
  - 输出扩展版 `metrics.csv`（含 FPS/帧时/超预算比例/降级触发等字段）。
  - 增加性能门禁判定（profile + 阈值）并在 `summary.md`/`summary.json` 给出结果。
  - 支持可选基线对比（与历史 CSV 比较，识别回退）。
- 补充脚本参数说明与输出说明。

### Out of Scope
- 在 `agent_world_viewer` 中新增 DrawCall、VRAM、GPU Pass 时间采集实现。
- 接入 Prometheus/OpenTelemetry 或外部时序存储系统。
- 修改 `testing-manual.md` 的大结构（该文档当前接近 500 行上限）。

## 接口 / 数据

### 输入参数（脚本）
- 新增/增强参数：
  - `--profile <web_desktop|high_density|stress>`：性能目标档位。
  - `--fps-target <n>`、`--fps-min <n>`、`--max-over-budget-pct <n>`、`--perf-budget-ms <n>`：门禁阈值覆盖。
  - `--baseline-csv <path>`、`--max-regression-pct <n>`：基线对比设置。

### 采集数据
- 来源：Viewer `perf_probe` 日志行（`viewer perf_probe ...`）。
- 关键字段：
  - `frame_ms_avg_last`
  - `frame_ms_p95_peak`
  - `over_budget_pct_peak`
  - `auto_degrade_seen`
  - `perf_samples`
  - 派生 `fps_avg_last`、`fps_p95_peak`

### 输出数据
- `metrics.csv`：每场景一行，包含性能指标与 gate 状态。
- `summary.md`：人类可读的阈值、结果与失败原因摘要。
- `summary.json`：机器可读的配置、逐场景指标、overall status。

## 里程碑
- M1：设计与项目管理文档建档。
- M2：`viewer-owr4-stress.sh` 完成指标采集、门禁判定、基线对比输出。
- M3：完成脚本语法/帮助/短窗冒烟验证并收口文档状态。

## 当前状态（2026-02-25）
- M1：已完成（建档完成）。
- M2：已完成（脚本已支持 profile 门禁、`perf_probe` 采集、`metrics.csv + summary.md + summary.json`、可选 baseline 对比与 `--enforce-gate`）。
- M3：已完成（`bash -n`、`--help`、短窗 smoke、baseline smoke 均已执行）。

## 风险
- 风险 1：运行环境波动导致短窗数据抖动，误判回退。
  - 缓解：门禁与基线对比采用可配置阈值；建议在固定机器上做发布口径对比。
- 风险 2：日志格式变化导致解析失败。
  - 缓解：对 `perf_probe` 解析提供缺失保护并显式标记 `perf_samples=0` 为失败。
- 风险 3：`llm_bootstrap` 在无 key 时退化为 script，影响横向可比性。
  - 缓解：输出 `mode` 字段并在摘要中保留 `script_fallback` 标识。
