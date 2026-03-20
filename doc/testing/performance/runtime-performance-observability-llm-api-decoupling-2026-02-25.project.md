# oasis7 Runtime：LLM API 延迟与代码执行耗时解耦（项目管理文档）

- 对应设计文档: `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.design.md`
- 对应需求文档: `doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.prd.md`

审计轮次: 4

## 任务拆解（含 PRD-ID 映射）
- [x] RPOF-L1 (PRD-TESTING-PERF-RPOFLLM-001/002): 完成设计文档与项目管理文档建档。
- [x] RPOF-L2 (PRD-TESTING-PERF-RPOFLLM-001/002/003): 实现 runtime perf 解耦（新增 `llm_api`、拆分 `decision_local`、保持 health/bottleneck 本地口径）。
- [x] RPOF-L3 (PRD-TESTING-PERF-RPOFLLM-001/002): 完成单测与高负载 + 真实 LLM 回归验证。
- [x] RPOF-L4 (PRD-TESTING-PERF-RPOFLLM-003): 文档与开发日志收口。
- [x] RPOF-L5 (PRD-TESTING-004): 专题文档按 strict schema 人工迁移并统一 `.prd.md/.project.md` 命名。

## 依赖
- doc/testing/performance/runtime-performance-observability-llm-api-decoupling-2026-02-25.prd.md
- `crates/oasis7/src/simulator/runtime_perf.rs`
- `crates/oasis7/src/simulator/runner.rs`
- `crates/oasis7/src/simulator/tests/runner.rs`
- `doc/testing/performance/runtime-performance-observability-foundation-2026-02-25.prd.md`
- `testing-manual.md`
- `doc/testing/prd.md`
- `doc/testing/project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 风险跟踪：trace 缺失 `llm_diagnostics.latency_ms` 时 `llm_api` 可能低估，需持续监控。
- 下一步：无（项目收口完成）
