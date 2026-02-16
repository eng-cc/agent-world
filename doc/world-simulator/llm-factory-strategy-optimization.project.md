# LLM 工厂闭环策略稳定性优化（项目管理文档）

## 任务拆解

### LFO0 文档立项
- [x] LFO0.1 输出设计文档（`doc/world-simulator/llm-factory-strategy-optimization.md`）
- [x] LFO0.2 输出项目管理文档（本文件）

### LFO1 Prompt 恢复规则（TODO-1）
- [x] LFO1.1 在 prompt 增加“失败原因 -> 下一动作”规则模板
- [x] LFO1.2 透出 `last_action` 结果摘要到 observation（含 reject reason）
- [x] LFO1.3 增加/更新 prompt 断言测试（拒绝原因驱动动作切换）

### LFO2 动作触发率统计（TODO-2）
- [x] LFO2.1 扩展 `world_llm_agent_demo` report 结构（`action_kind_counts` 等）
- [x] LFO2.2 在运行汇总阶段写入 `first_action_tick` 等关键字段
- [x] LFO2.3 增加 report JSON 结构回归测试

### LFO3 固定序列离线回归（TODO-3）
- [x] LFO3.1 新增 mock 决策序列测试（失败恢复 -> 建厂 -> 排产成功）
- [x] LFO3.2 新增反循环测试（限制连续无效 harvest）
- [x] LFO3.3 归并到 `test_tier_required` 并验证稳定通过

### LFO4 在线闭环验证与收口
- [x] LFO4.1 复跑 `llm_bootstrap --ticks 20` 并记录 `action_kind_counts`
- [x] LFO4.2 复跑 `llm_bootstrap --ticks 30` 对比策略收敛性
- [x] LFO4.3 回写文档状态、任务日志并按任务提交

### LFO4 在线闭环结果摘要（2026-02-16）
- 运行产物：
  - `output/llm_bootstrap/factory_schedule_lfo4_20_2026-02-16/run.log`
  - `output/llm_bootstrap/factory_schedule_lfo4_20_2026-02-16/report.json`
  - `output/llm_bootstrap/factory_schedule_lfo4_30_2026-02-16/run.log`
  - `output/llm_bootstrap/factory_schedule_lfo4_30_2026-02-16/report.json`
- 20 tick 结果：
  - `action_kind_counts`: `harvest_radiation=9`、`refine_compound=6`、`build_factory=1`、`schedule_recipe=4`
  - `action_kind_success_counts.schedule_recipe=3`
  - `first_action_tick`: `refine_compound=10`、`build_factory=11`、`schedule_recipe=12`
  - 结论：20 tick 内可完成“失败恢复 -> 建厂 -> 多次排产”。
- 30 tick 结果：
  - `action_kind_counts`: `harvest_radiation=19`、`refine_compound=5`、`build_factory=1`、`schedule_recipe=5`
  - `action_kind_success_counts.schedule_recipe=3`、`action_kind_failure_counts.schedule_recipe=2`
  - `first_action_tick` 与 20 tick 一致（`10/11/12`）。
  - 结论：长时窗口仍存在“电力不足后回退到连续 harvest”现象，后半程调度稳定性有待继续优化。
- 下一步优化建议（进入后续迭代）：
  - 增加“电力不足 -> harvest 上限轮次 + 立即回切 schedule”硬约束，避免尾段长 harvest。
  - 在 LLM 目标中加入“每 N tick 至少一次 schedule_recipe”的节奏约束，并结合 `action_kind_counts` 做回归门槛。

## 依赖
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `crates/agent_world/src/bin/world_llm_agent_demo.rs`
- `crates/agent_world/src/simulator/tests/kernel.rs`

## 状态
- 当前阶段：LFO0-LFO4 全部完成。
- 下一步：基于 LFO4 指标继续迭代策略约束（重点处理长窗口 harvest 回退）。
- 最近更新：2026-02-16（完成 LFO4 在线复跑与指标对比收口）。
