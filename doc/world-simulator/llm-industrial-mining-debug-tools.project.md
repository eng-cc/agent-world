# LLM 工业采矿闭环与调试补给工具（项目管理文档）

## 任务拆解

### MMD0 文档立项
- [x] MMD0.1 输出设计文档（`doc/world-simulator/llm-industrial-mining-debug-tools.md`）
- [x] MMD0.2 输出项目管理文档（本文件）

### MMD1 机制正确版（采矿 -> 精炼 -> 生产）
- [x] MMD1.1 扩展 `ResourceKind`：新增 `compound`
- [x] MMD1.2 扩展 `Action`：新增 `mine_compound`
- [x] MMD1.3 扩展经济参数：采矿电力成本/单次上限/单 location 上限
- [x] MMD1.4 实现 kernel 采矿执行（fragment 预算扣减 + compound 入账）
- [x] MMD1.5 升级 `refine_compound`：必须消耗 `compound`
- [x] MMD1.6 扩展事件/replay 与相关回归测试
- [x] MMD1.7 调整 `llm_bootstrap` 初始资源口径，验证“不能开局直建厂”

### MMD2 Debug 模式 LLM 补给工具
- [x] MMD2.1 新增配置开关 `AGENT_WORLD_LLM_DEBUG_MODE`（默认关闭）
- [x] MMD2.2 新增 `debug_grant_resource` 决策与 `Action::DebugGrantResource`
- [x] MMD2.3 OpenAI tools 仅在 debug 模式暴露 `agent_debug_grant_resource`
- [x] MMD2.4 非 debug 模式补给决策硬拒绝（解析/守卫）
- [x] MMD2.5 补齐 tool/schema/parser/behavior 单测

### MMD3 闭环验证与收口
- [x] MMD3.1 跑 `test_tier_required` 相关测试集
- [x] MMD3.2 运行 `llm_bootstrap` 在线闭环抽样，验证先采矿再生产
- [x] MMD3.3 回写文档状态与 devlog，提交收口

### MMD4 用户指令闭环复跑（2026-02-17）
- [x] MMD4.1 按用户目标执行在线闭环抽样（“所有工厂 + 所有制成品”）并保存 `run.log/report.json`
- [x] MMD4.2 基于首轮结果迭代 prompt 再抽样，验证是否提升 recipe 覆盖
- [x] MMD4.3 从两轮日志提取 recipe 级成功计数，回写 TODO 与项目状态

### MMD5 TODO-10~13 优化迭代（2026-02-17）
- [x] MMD5.1 文档增量设计：补充 TODO-10/11/12/13 优化目标、接口与风险
- [x] MMD5.2 守卫优化 A：`schedule_recipe` 位置前置检查 + `move_agent` 分段规划
- [x] MMD5.3 守卫优化 B：恢复链路与 `mine_compound` 单次上限对齐
- [x] MMD5.4 覆盖优化：recipe 覆盖记忆与硬切换策略
- [x] MMD5.5 回归验证：`test_tier_required` + `llm_bootstrap` 在线抽样复核
- [x] MMD5.6 文档收口：状态更新、TODO 回填、devlog 记录

### MMD6 TODO-14~16 优化迭代（2026-02-17）
- [x] MMD6.1 文档增量设计：补充 TODO-14/15/16 目标、接口与风险
- [x] MMD6.2 守卫优化 A：`factory_id` 归一化 + `build_factory` 去重改写
- [x] MMD6.3 守卫优化 B：`move_distance_exceeded` 目标记忆与不可见目标分段回退
- [x] MMD6.4 守卫优化 C：采矿耗尽感知（可采量记忆 + 质量裁剪 + 迁移回退）
- [x] MMD6.5 回归验证：`test_tier_required` + `llm_bootstrap` 在线抽样复核 + 文档收口

### MMD7 用户指令闭环复跑（2026-02-17）
- [x] MMD7.1 使用自定义强化 prompt 复跑 `llm_bootstrap --ticks 120`，验证“所有工厂 + 所有制成品 + 覆盖后持续产出”
- [x] MMD7.2 提取工厂/recipe 成功计数与失败分布，回填 TODO 与项目状态

### MMD4 结果摘要（2026-02-17）
- 运行 #1（100 tick，基线强化 prompt）：
  - 产物：
    - `output/llm_bootstrap/user_all_factories_all_products_2026-02-17_142739/run.log`
    - `output/llm_bootstrap/user_all_factories_all_products_2026-02-17_142739/report.json`
  - 指标：`action_success=69`、`action_failure=31`、`parse_errors=0`、`llm_errors=0`
  - recipe 成功计数（来自 `run.log`）：
    - `recipe.assembler.control_chip=3`
    - `recipe.assembler.motor_mk1=2`
    - `recipe.assembler.logistics_drone=0`
- 运行 #2（120 tick，先 `logistics_drone` 的顺序约束 prompt）：
  - 产物：
    - `output/llm_bootstrap/user_all_factories_all_products_retry_2026-02-17_143342/run.log`
    - `output/llm_bootstrap/user_all_factories_all_products_retry_2026-02-17_143342/report.json`
  - 指标：`action_success=79`、`action_failure=41`、`parse_errors=0`、`llm_errors=0`
  - recipe 成功计数（来自 `run.log`）：
    - `recipe.assembler.logistics_drone=1`
    - `recipe.assembler.motor_mk1=0`
    - `recipe.assembler.control_chip=0`
- 结论：
  - 两轮都未达成“三配方全覆盖”，但分别覆盖了不同配方，验证了“prompt 可影响配方偏置，但仍会被路径/守卫问题打断”。

### MMD5 验证摘要（2026-02-17）
- 运行 #1（60 tick，coverage-aware prompt）：
  - 产物：
    - `output/llm_bootstrap/mmd5_opt_all_recipes_fast_2026-02-17_155401/run.log`
    - `output/llm_bootstrap/mmd5_opt_all_recipes_fast_2026-02-17_155401/report.json`
  - 指标：`action_success=47`、`action_failure=13`、`parse_errors=0`、`llm_errors=0`
  - recipe 成功计数（来自 `run.log`）：
    - `recipe.assembler.control_chip=1`
    - `recipe.assembler.motor_mk1=1`
    - `recipe.assembler.logistics_drone=0`
- 运行 #2（120 tick，顺序约束 + 禁止重复 build prompt）：
  - 产物：
    - `output/llm_bootstrap/mmd5_opt_all_recipes_retry_2026-02-17_155732/run.log`
    - `output/llm_bootstrap/mmd5_opt_all_recipes_retry_2026-02-17_155732/report.json`
  - 指标：`action_success=91`、`action_failure=29`、`parse_errors=0`、`llm_errors=0`
  - recipe 成功计数（来自 `run.log`）：
    - `recipe.assembler.control_chip=2`
    - `recipe.assembler.motor_mk1=2`
    - `recipe.assembler.logistics_drone=1`
- 结论：
  - 在 120 tick 长窗口下，已达成三配方全覆盖（TODO-10/11/12/13 优化有效）。
  - 当前仍存在少量路径/资源抖动失败（`move_distance_exceeded`、`facility_not_found`、`insufficient_resource`），需在收口阶段整理为新 TODO。

### MMD6 验证摘要（2026-02-17）
- 运行 #1（120 tick 目标，守卫增强后在线抽样）：
  - 产物：
    - `output/llm_bootstrap/mmd6_opt_all_recipes_2026-02-17_162515/run.log`
    - `output/llm_bootstrap/mmd6_opt_all_recipes_2026-02-17_162515/report.json`
  - 指标：
    - `ticks_requested=120`、`active_ticks=58`
    - `action_success=46`、`action_failure=9`
    - `parse_errors=0`、`llm_errors=0`
    - `action_reject_reason_counts`：仅剩 `agent_already_at_location=2`、`insufficient_resource=7`
    - `facility_not_found=0`、`move_distance_exceeded=0`
  - recipe 成功计数（来自 `run.log`）：
    - `recipe.assembler.control_chip=2`
    - `recipe.assembler.motor_mk1=1`
    - `recipe.assembler.logistics_drone=1`
- 结论：
  - TODO-14/15/16 优化后，`facility_not_found` 和 `move_distance_exceeded` 在本轮样本中已清零。
  - 三配方覆盖在 58 active ticks 内达成；但策略在达成短期目标后出现 `wait/wait_ticks` 提前收敛，导致未跑满 120 tick。

### MMD7 验证摘要（2026-02-17，用户指令“所有工厂和制成品”）
- 运行 #1（120 tick，full-coverage + no-wait prompt）：
  - 产物：
    - `output/llm_bootstrap/user_all_factory_all_finished_codex_2026-02-17_170005/run.log`
    - `output/llm_bootstrap/user_all_factory_all_finished_codex_2026-02-17_170005/report.json`
  - 指标：
    - `ticks_requested=120`、`active_ticks=120`
    - `action_success=106`、`action_failure=13`
    - `parse_errors=1`、`llm_errors=0`
    - `action_reject_reason_counts={insufficient_resource:12, facility_not_found:1}`
  - 工厂建造成功计数（`run.log`）：
    - `factory.power.radiation.mk1=1`
    - `factory.assembler.mk1=1`
  - recipe 成功计数（`run.log`）：
    - `recipe.assembler.control_chip=1`
    - `recipe.assembler.motor_mk1=2`
    - `recipe.assembler.logistics_drone=1`
- 结论：
  - 已达成“所有工厂 + 所有制成品”目标，且 `active_ticks=120`，无提前收敛。
  - 仍出现 1 次协议解析问题（`tick=119` 多段输出 `---` 且 `schedule_recipe.batches=0` 导致 `parse_error`），以及 12 次资源不足拒绝。

## 依赖
- `crates/agent_world/src/simulator/types.rs`
- `crates/agent_world/src/simulator/world_model.rs`
- `crates/agent_world/src/simulator/kernel/actions.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/kernel/replay.rs`
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/simulator/llm_agent/openai_payload.rs`
- `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- `crates/agent_world/scenarios/llm_bootstrap.json`

## 状态
- 当前阶段：MMD7 已完成（用户目标“所有工厂 + 所有制成品”在线闭环达成）。
- 下一阶段：推进 TODO-17~TODO-20，优先收口“多段输出 + 非法 batches + 资源预检”链路。
- 最近更新：2026-02-17（完成 MMD7 在线复跑与 TODO 回填）。

## 遗留 TODO（产品优化）
- TODO-10~TODO-13：已完成（MMD5），并在 120 tick 在线抽样中验证三配方全覆盖。
- TODO-14~TODO-16：已完成（MMD6），并在在线抽样中验证关键失败项下降（`facility_not_found=0`、`move_distance_exceeded=0`）。
- TODO-17：将“覆盖后持续产出”从 prompt 约束下沉为默认策略（本轮通过强化 prompt 实现 `active_ticks=120`，但默认策略仍未固化）。
- TODO-18：tool-only 协议仍会出现多段输出（`---`）+ 非法 `schedule_recipe.batches=0` 组合，需增加“多段输出硬拒绝 + 非法 batches 自动改写（mine/refine）”。
- TODO-19：`schedule_recipe` 与 `move_agent` 的电力前置检查仍有缺口（本轮 `insufficient_resource=12`），需在 guardrail 增加“动作前电力预算校验 + 自动回切 harvest”。
- TODO-20：矿点耗尽后仍可能出现重复高质量采矿尝试、失败后再恢复的抖动路径，需补“耗尽点冷却/跳过窗口”降低无效失败。
