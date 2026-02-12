# OWR4 压测结果摘要

- 运行目录：`artifacts/owr4_baseline/20260212-130428`
- 运行时长（每场景）：`12` 秒
- Tick 间隔：`200` ms
- 参考模板：`doc/world-simulator/viewer-open-world-sandbox-readiness.stress-report.template.md`

| Scenario | Mode | Duration(s) | Tick(ms) | Final Events | Events/s | Viewer Status |
|---|---:|---:|---:|---:|---:|---|
| triad_region_bootstrap | script | 12 | 200 | 30 | 2.50 | connected |
| llm_bootstrap | script_fallback(no_openai_key) | 12 | 200 | 32 | 2.67 | connected |
