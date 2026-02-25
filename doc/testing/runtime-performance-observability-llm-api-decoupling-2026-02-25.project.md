# Agent World Runtime：LLM API 延迟与代码执行耗时解耦（项目管理）

## 任务拆解
- [x] `RPOF-L1`：设计/项目文档建档
  - [x] 新建设计文档 `runtime-performance-observability-llm-api-decoupling-2026-02-25.md`
  - [x] 新建项目管理文档并确认任务边界
- [ ] `RPOF-L2`：实现 runtime perf 解耦
  - [ ] `runtime_perf` 增加 `llm_api` 序列与记录接口
  - [ ] `runner` 在 `tick/tick_decide_only` 拆分 `decision_local` 与 `llm_api`
  - [ ] 保持 `health/bottleneck` 仅基于本地代码执行阶段
- [ ] `RPOF-L3`：测试与高负载验证
  - [ ] 单测：`runtime_perf` 新增边界断言
  - [ ] 单测：`runner` decision/llm_api 拆分断言
  - [ ] 回归：高 action + 真实 LLM 场景探针
- [ ] `RPOF-L4`：文档收口
  - [ ] 更新本项目状态
  - [ ] 更新 `doc/devlog/2026-02-25.md`

## 依赖
- 设计文档：`doc/testing/runtime-performance-observability-llm-api-decoupling-2026-02-25.md`
- 已有 runtime perf 基础：`doc/testing/runtime-performance-observability-foundation-2026-02-25.md`
- 测试手册：`testing-manual.md`
- 关键代码：
  - `crates/agent_world/src/simulator/runtime_perf.rs`
  - `crates/agent_world/src/simulator/runner.rs`
  - `crates/agent_world/src/simulator/tests/runner.rs`

## 状态
- 当前阶段：`RPOF-L1` 完成，`RPOF-L2` 进行中
- 阻塞：无
- 风险跟踪：
  - trace 缺失 `llm_diagnostics.latency_ms` 时会造成 `llm_api` 低估
  - 下游脚本需兼容新增字段
- 最近更新：2026-02-25
