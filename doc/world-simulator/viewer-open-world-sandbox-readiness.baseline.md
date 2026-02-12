# Viewer OWR4 跨版本基线（2026-02-12）

## 基线元信息
- 基线日期：2026-02-12
- 基线代码：`24dcaa3`（在该提交基础上执行 OWR4.5 基线采样）
- 压测目录：`artifacts/owr4_baseline/20260212-130428`
- 说明：当前环境未提供 `OPENAI_API_KEY`，`llm_bootstrap` 使用 `script_fallback(no_openai_key)` 模式采样。

## 指标口径
- `events/s`：来自 `scripts/viewer-owr4-stress.sh`（headless+autoplay）统计的事件窗口增长速率。
- `frame_ms_avg/p95`、`over_budget_pct`：来自 `AGENT_WORLD_VIEWER_PERF_PROBE=1` 的 viewer 日志（`budget_ms=33`）。
- `over_budget_pct` 作为“卡顿/超预算帧占比”指标。

## 当前基线
| 场景 | mode | tick(ms) | duration(s) | events/s | frame_ms_avg | frame_ms_p95 | over_budget_pct | event_window_size(末次) | 备注 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| `triad_region_bootstrap` | script | 200 | 12 | 2.50 | 17.13 | 22.47 | 2.85% | 52 | `auto_degrade=true` |
| `llm_bootstrap` | script_fallback(no_openai_key) | 200 | 12 | 2.67 | 17.78 | 21.43 | 1.38% | 49 | `auto_degrade=true` |

## 原始数据定位
- 吞吐数据：`artifacts/owr4_baseline/20260212-130428/metrics.csv`
- triad perf 日志：`artifacts/owr4_baseline/20260212-130428/frame_probe/triad_viewer.log`
- llm perf 日志：`artifacts/owr4_baseline/20260212-130428/frame_probe/llm_viewer.log`
- triad 截图：`artifacts/owr4_baseline/20260212-130428/frame_probe/triad_window.png`
- llm 截图：`artifacts/owr4_baseline/20260212-130428/frame_probe/llm_window.png`

## 后续对比模板
| 版本 | 场景 | events/s Δ | frame_ms_p95 Δ | over_budget_pct Δ | 结论 |
|---|---|---:|---:|---:|---|
| 待填 | triad_region_bootstrap |  |  |  |  |
| 待填 | llm_bootstrap |  |  |  |  |
