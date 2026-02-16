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
- [ ] LFA3.1 新增/更新 kernel 行为单测（建厂、排产、拒绝路径）
- [ ] LFA3.2 跑 `test_tier_required` 口径回归
- [ ] LFA3.3 复跑 `llm_bootstrap` 在线闭环并记录结果
- [ ] LFA3.4 回写文档状态、任务日志并完成收口提交

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
- 当前阶段：LFA0-LFA2 完成，LFA3 进行中。
- 下一步：执行 LFA3（在线闭环验证与收口）。
- 最近更新：2026-02-16（完成 LFA2：decision 解析、prompt schema、观测资源透出与测试更新）。
