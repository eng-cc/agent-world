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

### LFO5 用户指令闭环抽样（2026-02-16）
- [x] LFO5.1 使用“建工厂 + 制成品”硬目标 prompt 复跑 `llm_bootstrap --ticks 30`（含 `--print-llm-io`）
- [x] LFO5.2 验证主链路达成：`build_factory` 成功且 `schedule_recipe` 至少 2 次成功，`data` 正增长
- [x] LFO5.3 记录本轮新发现 TODO 并回写项目文档/任务日志

### LFO6 用户指令 TODO 收口（2026-02-16）
- [x] LFO6.1 修复 `facility_already_exists` 在 prompt `last_action.reject_reason` 被降级为 `other` 的问题
- [ ] LFO6.2 收敛 `execute_until` 尾段长 harvest（增加 harvest 连续轮次硬上限与回切提示）
- [ ] LFO6.3 增加 `schedule_recipe.batches` 与可用硬件上界约束（含 guardrail 与提示文案）

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

### LFO5 在线闭环结果摘要（2026-02-16，用户指令复跑）
- 运行产物：
  - `output/llm_bootstrap/user_factory_closedloop_2026-02-16_230752/run.log`
  - `output/llm_bootstrap/user_factory_closedloop_2026-02-16_230752/report.json`
- 关键指标：
  - `action_success=27`、`action_failure=3`、`llm_errors=0`、`parse_errors=0`
  - `action_kind_counts={harvest_radiation:18, refine_compound:5, build_factory:2, schedule_recipe:5}`
  - `action_kind_success_counts.schedule_recipe=3`
  - `first_action_tick={refine_compound:10, build_factory:11, schedule_recipe:12}`
- 链路确认（run.log 证据）：
  - `tick=11` 建厂成功（`build_factory`）。
  - `tick=14`、`tick=16`、`tick=18` 三次排产成功（`schedule_recipe`）。
  - `data` 由初始 `12` 增长到 `24`，已达成“建工厂 + 制成品（data）”闭环。
- 新增产品 TODO（进入下一轮优化）：
  - TODO-1：`facility_already_exists` 在下一轮 observation 被降级为 `reject_reason=other`，导致模型缺少可恢复语义；需要补 reject_reason 枚举透传映射。
  - TODO-2：尾段仍出现 `execute_until` 长 harvest（`tick=19..27`），建议加入“能量补足后立即退出 execute_until 并优先回切排产”约束。
  - TODO-3：模型会在硬件不足时发出 `batches=2` 的排产（`tick=30`，请求16仅有7）；建议在 prompt 增加 `batches <= available_hardware / recipe_hardware_cost_per_batch` 的上界规则。

## 依赖
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `crates/agent_world/src/bin/world_llm_agent_demo.rs`
- `crates/agent_world/src/simulator/tests/kernel.rs`

## 状态
- 当前阶段：LFO0-LFO5 与 LFO6.1 已完成；LFO6.2-LFO6.3 进行中。
- 下一步：完成 LFO6.2/LFO6.3，并执行在线复跑确认 TODO 收口效果。
- 最近更新：2026-02-16（完成 LFO6.1 reject_reason 透传修复）。
