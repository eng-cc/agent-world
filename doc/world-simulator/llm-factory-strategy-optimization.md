# LLM 工厂闭环策略稳定性优化（llm_bootstrap）

## 目标
- 在 `llm_bootstrap`（20 tick 默认口径）下，将“可建厂但未稳定排产”的问题从能力缺口转为可度量、可回归的策略问题。
- 引入“失败原因 -> 下一动作”强规则，减少无效重复 `harvest_radiation`，推动动作链路稳定收敛到 `refine_compound -> build_factory -> schedule_recipe`。
- 增强闭环可观测性：报告中直接统计动作触发率与关键动作首达 tick。
- 建立固定 mock 输出序列回归，保证策略主链路可在离线测试稳定复现。

## 范围
- In Scope:
  - 在 LLM prompt 组装中新增拒绝原因恢复规则模板（Recovery Policy），并将最近拒绝原因结构化透出给模型。
  - 在 `world_llm_agent_demo` 报告增加动作级指标（`action_kind_counts` 等），用于量化建厂/排产触发率。
  - 新增 deterministic mock-sequence 回归，覆盖“资源不足恢复 -> 建厂 -> 排产”成功链路。
  - 补充 `test_tier_required` 单测与最小在线复跑口径。
- Out of Scope:
  - 不切换到 runtime M4 完整生产链路。
  - 不引入模型训练、RL 或外部策略服务。
  - 不改动 viewer 渲染层。

## 接口 / 数据

### 1) Prompt 恢复规则（TODO-1）
- 位置：`crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- 设计：
  - 在“决策约束”段新增固定恢复规则表（文本模板 + 可枚举 reject reason）。
  - 明确“当上一步因资源拒绝时，下一个动作必须是补资源动作；禁止连续超过 N 次相同采集动作”。
- 规则初版（建议）：
  - `insufficient_resource.hardware -> refine_compound`（`compound_mass_g` 下限由模板给出）
  - `insufficient_resource.electricity -> harvest_radiation`
  - `factory_not_found -> build_factory`
  - `agent_already_at_location -> schedule_recipe | refine_compound`（禁止重复 move）
- 数据透出：
  - 在 prompt observation 中新增最近一次动作结果摘要：
    - `last_action.kind`
    - `last_action.success`
    - `last_action.reject_reason`

### 2) 报告动作触发率指标（TODO-2）
- 位置：`crates/agent_world/src/bin/world_llm_agent_demo.rs`（及其 report 结构）
- 新增字段（JSON）：
  - `action_kind_counts: { "<action_kind>": <u64> }`
  - `action_kind_success_counts: { "<action_kind>": <u64> }`
  - `action_kind_failure_counts: { "<action_kind>": <u64> }`
  - `first_action_tick: { "<action_kind>": <u64|null> }`
- 最小验收指标：
  - `build_factory`、`schedule_recipe` 必须可在报告中独立统计。
  - 可通过报告快速判断“是否触发过建厂/排产、首达在第几 tick”。

### 3) 固定序列离线回归（TODO-3）
- 位置：
  - `crates/agent_world/src/simulator/llm_agent/tests.rs`
  - `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
  - 如需新文件：`crates/agent_world/src/simulator/llm_agent/tests_strategy.rs`
- 用例设计：
  - 用例 A（成功主链路）：
    - mock 决策序列：`build_factory`（失败: insufficient hardware）-> `refine_compound` -> `build_factory`（成功）-> `schedule_recipe`（成功）。
    - 断言：最终存在工厂，`data` 增长，事件包含 `FactoryBuilt`/`RecipeScheduled`。
  - 用例 B（反循环约束）：
    - mock 连续 `harvest_radiation` 超阈值后，验证重规划约束触发并切换动作建议。
    - 断言：不会无限重复同一无效动作签名。

## 里程碑
- LFO0：设计与项目文档立项。
- LFO1：Prompt 恢复规则接线 + 相关单测。
- LFO2：报告动作指标接线 + JSON 回归断言。
- LFO3：固定序列离线回归补齐（成功链路/反循环）。
- LFO4：在线 `llm_bootstrap` 复跑，更新结果与阈值（20 tick 与 30 tick 对比）。

## 风险
- 规则过强风险：过度硬编码可能压制模型在复杂场景的探索能力。
- 指标兼容风险：报告 JSON 新字段需保持向后兼容（新增不破坏旧消费方）。
- 测试漂移风险：离线 mock 策略与在线真实模型行为仍可能存在差异，需双口径保留（离线确定性 + 在线抽样）。
