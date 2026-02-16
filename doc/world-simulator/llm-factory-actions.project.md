# LLM 工厂动作接入（项目管理文档）

## 任务拆解

### LFA0 文档立项
- [x] LFA0.1 输出设计文档（`doc/world-simulator/llm-factory-actions.md`）
- [x] LFA0.2 输出项目管理文档（本文件）

### LFA1 simulator 动作与状态
- [x] LFA1.1 扩展 `Action`：`BuildFactory`、`ScheduleRecipe`
- [x] LFA1.2 扩展 `WorldModel`：新增 `factories` 状态
- [x] LFA1.3 扩展 `WorldEventKind`：`FactoryBuilt`、`RecipeScheduled`
- [x] LFA1.4 实现 kernel 执行与 replay 应用

### LFA2 LLM 接线与 Prompt
- [x] LFA2.1 扩展 decision 解析：`build_factory`、`schedule_recipe`
- [x] LFA2.2 更新 prompt schema 与推荐模板
- [x] LFA2.3 更新相关断言测试（prompt/parse）

### LFA3 闭环验证与收口
- [x] LFA3.1 新增/更新 kernel 行为单测（建厂、排产、拒绝路径）
- [x] LFA3.2 跑 `test_tier_required` 口径回归
- [x] LFA3.3 复跑 `llm_bootstrap` 在线闭环并记录结果
- [x] LFA3.4 回写文档状态、任务日志并完成收口提交

### LFA3 在线闭环结果摘要（2026-02-16）
- 运行产物：
  - `output/llm_bootstrap/factory_schedule_2026-02-16/run.log`
  - `output/llm_bootstrap/factory_schedule_2026-02-16/report.json`
  - `output/llm_bootstrap/factory_schedule_forced_2026-02-16/run.log`
  - `output/llm_bootstrap/factory_schedule_forced_2026-02-16/report.json`
- 关键观察：
  - 新动作已被在线模型真实调用：`build_factory`（并触发内核资源校验拒绝）。
  - 在默认 20 tick 下，模型策略仍偏向持续 `harvest_radiation`，未稳定切换到 `refine_compound -> build_factory -> schedule_recipe` 完整链路。
- 产品 TODO（后续优化）：
  - TODO-1：在 prompt 中加入“失败原因 -> 下一动作”强规则模板（如 `insufficient hardware -> refine_compound`）。
  - TODO-2：在 `world_llm_agent_demo` 报告增加 `action_kind_counts`，便于量化建厂/排产动作实际触发率。
  - TODO-3：补充一个可控离线策略回归（固定 mock 输出序列）覆盖 `build_factory -> schedule_recipe` 端到端成功路径。

## 依赖
- `crates/agent_world/src/simulator/types.rs`
- `crates/agent_world/src/simulator/world_model.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/kernel/actions.rs`
- `crates/agent_world/src/simulator/kernel/replay.rs`
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `crates/agent_world/src/simulator/tests/kernel.rs`

## 状态
- 当前阶段：LFA0-LFA3 全部完成。
- 下一步：按 TODO-1~TODO-3 进入策略与可观测性优化迭代（见 `doc/world-simulator/llm-factory-strategy-optimization.md` 与 `doc/world-simulator/llm-factory-strategy-optimization.project.md`）。
- 最近更新：2026-02-16（完成 LFA3 收口并迁移后续任务到 LFO 文档）。
