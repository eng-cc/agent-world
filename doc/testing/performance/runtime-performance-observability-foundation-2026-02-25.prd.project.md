# Agent World Runtime：代码执行性能采集/统计/分析基础（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] RPOF-1 (PRD-TESTING-PERF-RPOF-001): 完成基础设计与项目管理建档。
- [x] RPOF-2 (PRD-TESTING-PERF-RPOF-001/002): 实现 runtime perf 模块（采集、统计、分析）与 health/bottleneck 规则。
- [x] RPOF-3 (PRD-TESTING-PERF-RPOF-001/002): 接入 `AgentRunner` 与 `RunnerMetrics`，覆盖 `tick/tick_decide_only/notify_action_result` 与外部 action 补录。
- [x] RPOF-4 (PRD-TESTING-PERF-RPOF-002/003): 打通 `world_llm_agent_demo`、`world_viewer_live` 输出链路。
- [x] RPOF-5 (PRD-TESTING-PERF-RPOF-003): 扩展 `scripts/llm-longrun-stress.sh` 汇总 runtime perf 字段。
- [x] RPOF-6 (PRD-TESTING-PERF-RPOF-001/002/003): 完成单测、脚本回归与 `cargo check` 收口。
- [x] RPOF-7 (PRD-TESTING-PERF-RPOF-003): 回写文档状态与 devlog。
- [x] RPOF-8 (PRD-TESTING-004): 专题文档按 strict schema 人工迁移并统一 `.prd.md/.prd.project.md` 命名。

## 依赖
- `crates/agent_world/src/simulator/runner.rs`
- `crates/agent_world/src/bin/world_llm_agent_demo/*`
- `crates/agent_world/src/viewer/live_*`
- `scripts/llm-longrun-stress.sh`
- `testing-manual.md`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 风险跟踪：`tick_decide_only` 路径的外部 action 补录仍需在后续改动中持续守护。
- 下一步：无（项目收口完成）
