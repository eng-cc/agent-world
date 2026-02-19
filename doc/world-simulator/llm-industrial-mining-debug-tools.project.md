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

### MMD8 TODO-17~20 收口迭代（2026-02-17）
- [x] MMD8.1 文档增量设计：补充 TODO-17/18/19/20 目标、接口与风险
- [x] MMD8.2 协议收敛：单轮多 tool call 硬拒绝 + `schedule_recipe.batches<=0` 自动归一
- [x] MMD8.3 资源预检：`schedule_recipe` 与 `move_agent` 电力前置预算 guardrail
- [x] MMD8.4 策略收敛：覆盖后 wait 自动回切持续产出 + 采矿耗尽冷却窗口
- [x] MMD8.5 回归验证：`test_tier_required` + `llm_bootstrap` 在线抽样 + 文档收口

### MMD9 TODO-22~25 收敛迭代（2026-02-17）
- [x] MMD9.1 `execute_until` 动作匹配补齐（覆盖 `mine_compound/refine_compound`）+ `test_tier_required` 回归
- [x] MMD9.2 `schedule_recipe` 工厂-配方兼容校验（assembler recipe 仅允许 assembler factory）+ kernel 回归
- [x] MMD9.3 采矿失败记忆优先级（候选排序避开连续失败矿点）+ `test_tier_required` 回归
- [x] MMD9.4 guardrail 改写后 `execute_until.until` 条件重建（抑制后半程 move/mine 抖动）+ 在线抽样复核

### MMD11 TODO-26~29 收敛迭代（2026-02-19）
- [x] MMD11.1 文档增量设计：补充 TODO-26/27/28/29 目标、接口与风险
- [x] MMD11.2 guardrail 修复：`schedule_recipe/build_factory` 的 factory/location 归一与兼容改写
- [x] MMD11.3 回归验证：`test_tier_required` 定向单测与 llm_agent 全量回归
- [x] MMD11.4 在线复验：`llm_bootstrap --ticks 120` 对照复跑并回写文档/devlog

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

### MMD8.4 实施摘要（2026-02-17）
- 范围：
  - TODO-17：`wait/wait_ticks` 在三配方全覆盖后自动改写为持续产出动作（优先 `schedule_recipe`，不可执行时沿现有 guardrail 自动回切恢复动作）。
  - TODO-20：为矿点耗尽新增 location 级冷却窗口，冷却期内跳过原矿点并优先替代矿点；冷却过期后允许重试。
- 代码：
  - `crates/agent_world/src/simulator/llm_agent.rs`
  - `crates/agent_world/src/simulator/llm_agent/behavior_loop.rs`
  - `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- 单测：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent_rewrites_wait --features test_tier_required -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent_skips_depleted_location_during_cooldown_window --features test_tier_required -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent_allows_retry_depleted_location_after_cooldown_expires --features test_tier_required -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent --features test_tier_required -- --nocapture`

### MMD8.5 验证摘要（2026-02-17）
- required-tier 回归：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent --features test_tier_required -- --nocapture`
  - 结果：`llm_agent` 相关 120 个用例通过（含 MMD8.4 新增“覆盖后 wait 回切”和“采矿冷却窗口”用例）。
- 在线闭环抽样（120 tick，目标“所有工厂 + 所有制成品 + 覆盖后持续产出”）：
  - 命令：`AGENT_WORLD_LLM_SYSTEM_PROMPT='你是工业闭环调度Agent。必须通过单一tool call输出一个可执行决策，禁止额外文本。主目标：完成所有工厂类型与所有制成品配方，并在覆盖后持续产出。' AGENT_WORLD_LLM_SHORT_TERM_GOAL='120 tick 内至少完成：1) build_factory(factory.power.radiation.mk1) 成功>=1；2) build_factory(factory.assembler.mk1) 成功>=1；3) schedule_recipe(recipe.assembler.control_chip) 成功>=1；4) schedule_recipe(recipe.assembler.motor_mk1) 成功>=1；5) schedule_recipe(recipe.assembler.logistics_drone) 成功>=1。恢复规则：insufficient_resource.electricity -> harvest_radiation；insufficient_resource.hardware -> mine_compound + refine_compound；factory_not_found -> build_factory 或移动到已知工厂；facility_already_exists -> 切换未覆盖 recipe；采矿点若可采为0则切换替代矿点。' AGENT_WORLD_LLM_LONG_TERM_GOAL='形成并保持持续工业链：harvest_radiation -> mine_compound -> refine_compound -> build_factory -> schedule_recipe；全覆盖后持续提升 data，不提前收敛。' env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_llm_agent_demo -- llm_bootstrap --ticks 120 --print-llm-io --llm-io-max-chars 1800 --report-json output/llm_bootstrap/mmd8_opt_all_factory_all_products_2026-02-17_175323/report.json`
  - 产物：
    - `output/llm_bootstrap/mmd8_opt_all_factory_all_products_2026-02-17_175323/run.log`
    - `output/llm_bootstrap/mmd8_opt_all_factory_all_products_2026-02-17_175323/report.json`
  - 指标：
    - `ticks_requested=120`、`active_ticks=120`
    - `action_success=97`、`action_failure=23`
    - `parse_errors=0`、`llm_errors=0`、`repair_rounds_total=0`
    - `action_reject_reason_counts={insufficient_resource:18, agent_not_at_location:4, agent_already_at_location:1}`
  - 工厂建造成功计数（`run.log`）：
    - `factory.power.radiation.mk1=1`（首次成功 `tick=15`）
    - `factory.assembler.mk1=1`（首次成功 `tick=21`）
  - recipe 成功计数（`run.log`）：
    - `recipe.assembler.control_chip=2`（首次成功 `tick=22`）
    - `recipe.assembler.motor_mk1=1`（首次成功 `tick=45`）
    - `recipe.assembler.logistics_drone=1`（首次成功 `tick=77`）
  - 观测：
    - 全程 `decision_wait=0`、`decision_wait_ticks=0`，未出现旧的提前空转。
    - 日志出现冷却守卫命中记录（`mine_compound cooldown guardrail rerouted...`），验证 TODO-20 跳过窗口在在线样本中生效。

### MMD10 未收口稳定性项复现与评估（2026-02-19）
- 复现实验 #1（120 tick，完整口径）：
  - 命令：`AGENT_WORLD_LLM_SYSTEM_PROMPT='你是工业闭环调度Agent。每轮只输出一个可执行 tool call 对应的 decision，不输出解释文本。优先形成稳定工业闭环，避免在同一耗尽矿点连续重试。' AGENT_WORLD_LLM_SHORT_TERM_GOAL='120 tick 内完成：1) build_factory(factory.power.radiation.mk1) 成功>=1；2) build_factory(factory.assembler.mk1) 成功>=1；3) schedule_recipe 成功覆盖 recipe.assembler.control_chip / recipe.assembler.motor_mk1 / recipe.assembler.logistics_drone 各>=1。覆盖完成后继续排产，禁止 wait/wait_ticks。恢复规则：electricity不足->harvest_radiation；hardware不足->mine_compound+refine_compound；factory_not_found->build_factory。' AGENT_WORLD_LLM_LONG_TERM_GOAL='维持 harvest_radiation -> mine_compound -> refine_compound -> build_factory -> schedule_recipe 的持续产出闭环，减少 move/mine 抖动和无效重试。' env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_llm_agent_demo -- llm_bootstrap --ticks 120 --print-llm-io --llm-io-max-chars 1800 --report-json output/llm_bootstrap/repro_mmd9_2026-02-19_110325/report.json`
  - 产物：
    - `output/llm_bootstrap/repro_mmd9_2026-02-19_110325/run.log`
    - `output/llm_bootstrap/repro_mmd9_2026-02-19_110325/report.json`
  - 指标：
    - `ticks_requested=120`、`active_ticks=120`
    - `action_success=49`、`action_failure=71`
    - `action_reject_reason_counts={rule_denied:42, facility_not_found:25, location_not_found:3, agent_already_at_location:1}`
    - `llm_errors=0`、`parse_errors=0`、`repair_rounds_total=3`
    - `llm_input_chars_max=60197`、`prompt_section_clipped=380`
  - 观测：
    - TODO-26/TODO-27 均已高强度复现，且占总失败 `67/71`。
    - 同轮出现多段 `---` 输出（`run.log` 中 `---` 共 7 次），协议不稳定仍存在。
    - TODO-21 对应的 `insufficient_resource` 在本轮为 `0`，失败主因已切换到工厂选择/配方工厂匹配错误。
- 复现实验 #2（同口径二次复现，执行到 `tick=51` 后手动中断）：
  - 产物：`output/llm_bootstrap/repro_mmd9_b_2026-02-19_111015/run.log`
  - 观测：
    - 截止 `tick=51`，已出现 `rule_denied=12`、`facility_not_found=6`、`location_not_found=1`。
    - 与实验 #1 的失败模式一致，复现具有稳定性，不是单次偶发。
- 是否需要优化（结论）：
  - 需要，且应立即进入优化迭代。
  - 优先级建议：
    - P0：先收敛 TODO-26/TODO-27（工厂 ID 归一 + recipe/factory_kind 前置 guardrail），压降主失败源。
    - P1：补 location_id 归一与无效 location 拦截，收敛 `location_not_found`。
    - P2：继续推进多段输出协议约束与 Prompt 体积治理，降低 `---` 输出与高裁剪带来的行为漂移。

### MMD11 收口实施与在线复验（2026-02-19）
- MMD11.2/MMD11.3 代码与测试：
  - 代码：
    - `crates/agent_world/src/simulator/llm_agent/behavior_runtime_helpers.rs`
    - `crates/agent_world/src/simulator/llm_agent/behavior_guardrails.rs`
    - `crates/agent_world/src/simulator/llm_agent/behavior_prompt.rs`
    - `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
  - 测试：
    - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent_reroutes_schedule_recipe --features test_tier_required -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent_build_factory_normalizes_unknown_location_to_current_location --features test_tier_required -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent_user_prompt_preserves_ --features test_tier_required -- --nocapture`
    - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent --features test_tier_required -- --nocapture`
- MMD11.4 在线复验（120 tick，同 MMD10 口径）：
  - 产物：
    - `output/llm_bootstrap/mmd11_verify_2026-02-19_115519/run.log`
    - `output/llm_bootstrap/mmd11_verify_2026-02-19_115519/report.json`
  - 指标（对比 `repro_mmd9_2026-02-19_110325`）：
    - `action_failure`: `71 -> 10`
    - `rule_denied`: `42 -> 0`
    - `facility_not_found`: `25 -> 0`
    - `location_not_found`: `3 -> 10`（退化）
    - `parse_errors`: `0 -> 60`（显著退化）
    - `decision_wait`: `0 -> 60`（出现大面积空转）
    - `prompt_section_clipped`: `380 -> 1192`
    - `run.log` 中 `---` 次数：`7 -> 214`
  - 观测：
    - 工厂/配方不兼容拒绝（TODO-26/27 目标项）在本轮 reject_reason 统计中已清零；
    - 但协议噪声与 prompt 裁剪恶化，导致大量 `Wait` 与低有效动作占比，闭环目标未达成。
  - 结论：
    - TODO-26/TODO-27 进入“代码已落地、在线样本待二次确认”状态；
    - TODO-28/TODO-29 仍未收口，且优先级上升为 P0（先协议、再位置）。

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
- 当前阶段：MMD11.2~MMD11.4 已执行完成（含代码、required-tier 回归、在线复验）。
- 下一阶段：围绕 TODO-29/TODO-28 开启下一轮收敛（协议稳定性优先，随后复核 location 归一）。
- 最近更新：2026-02-19（完成 MMD11 实施与在线复验回写）。

## 遗留 TODO（产品优化）
- TODO-10~TODO-13：已完成（MMD5），并在 120 tick 在线抽样中验证三配方全覆盖。
- TODO-14~TODO-16：已完成（MMD6），并在在线抽样中验证关键失败项下降（`facility_not_found=0`、`move_distance_exceeded=0`）。
- TODO-17：已完成（MMD8.4），`wait/wait_ticks` 在 recipe 全覆盖后默认回切持续产出动作，不再仅依赖 prompt 约束。
- TODO-18：已完成（MMD8.2），单轮多 tool call 硬拒绝，且 `schedule_recipe.batches<=0` 自动归一为合法下界。
- TODO-19：已完成（MMD8.3），`schedule_recipe` 与 `move_agent` 增加电力前置预算预检并可自动回切 `harvest_radiation`。
- TODO-20：已完成（MMD8.4），矿点耗尽引入短期冷却/跳过窗口并支持冷却过期后重试。
- TODO-21：`mmd9_opt_all_factory_all_products_2026-02-18_115214` 中 `insufficient_resource=20/24`；但在 `repro_mmd9_2026-02-19_110325` 中未复现（`0`）。该问题暂降为次优先级，后续在收敛工厂选择误差后再复查。
- TODO-22：已完成（MMD9.1），`execute_until` 现可正确跟踪 `mine_compound/refine_compound` 的失败/拒绝，避免“失败后继续 auto step”。
- TODO-23：已完成（MMD9.2），kernel `schedule_recipe` 现会校验 recipe 所需工厂类型，已阻断 power factory 调度 assembler recipe。
- TODO-24：已完成（MMD9.3），新增矿点失败 streak 记忆并接入候选排序，优先避开连续失败矿点，降低耗尽矿点重复重试。
- TODO-25：已完成（MMD9.4），`execute_until.action` 经 guardrail 改写后会同步重建默认 `until` 条件，避免动作类型切换后沿用旧条件导致抖动。
- TODO-26：已在 MMD11.2 落地（factory_id 归一 + 缺厂改写 build_factory）；定向单测与 llm_agent required-tier 已通过。`mmd11_verify_2026-02-19_115519` 中 `facility_not_found=0`，但受高 parse_error 干扰，需在 TODO-29 收敛后做二次在线复核。
- TODO-27：已在 MMD11.2/MMD11.3 落地（recipe->factory_kind 前置校验 + 兼容工厂改写 + prompt 恢复策略）；定向单测与 llm_agent required-tier 已通过。`mmd11_verify_2026-02-19_115519` 中 `rule_denied=0`，同样需在协议稳定后复核持续性。
- TODO-28：未收口且出现退化；`location_not_found` 由基线 `3` 升至 `10`。需继续收敛“不可识别 location 直接回退当前 location”并减少模型生成伪 location alias。
- TODO-29：未收口且显著退化；`parse_errors=60`、`run.log` 中 `---=214`、`prompt_section_clipped=1192`。需优先推进“多段输出硬拒绝/末段决策提取 + prompt 体积治理”。
