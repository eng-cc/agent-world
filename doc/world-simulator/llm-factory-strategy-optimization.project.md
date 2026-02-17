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
- [x] LFO6.2 收敛 `execute_until` 尾段长 harvest（增加 harvest 连续轮次硬上限与回切提示）
- [x] LFO6.3 增加 `schedule_recipe.batches` 与可用硬件上界约束（含 guardrail 与提示文案）
- [x] LFO6.4 当 `schedule_recipe` 连单批也不可执行时，guardrail 自动回切 `refine_compound/harvest_radiation`
- [x] LFO6.5 基于 LFO6.1-LFO6.4 在线复跑 `llm_bootstrap --ticks 30` 并回填对比指标

### LFO7 决策全量 Tool 化（2026-02-17）
- [x] LFO7.1 输出增量设计文档并补充项目拆解（tool-only 协议）
- [x] LFO7.2 扩展 Responses tools：新增 `agent_submit_decision`，并切换 `tool_choice=required`
- [x] LFO7.3 调整 prompt/behavior 约束为“只允许 tool call 输出”，更新解析路径与单测
- [x] LFO7.4 跑通 required-tier 测试并记录 devlog/状态回填

### LFO8 Tool-only 在线抽样与 TODO 回填（2026-02-17）
- [x] LFO8.1 使用“建工厂 + 制成品”目标 prompt 复跑 `llm_bootstrap --ticks 30`，输出 JSON 报告
- [x] LFO8.2 使用 `--print-llm-io` 短程抽样定位高 `parse_errors` 根因
- [x] LFO8.3 回写 TODO 并更新项目文档/devlog

### LFO9 Tool-only 类型化输入收敛（2026-02-17）
- [x] LFO9.1 更新增量设计文档与项目拆解（删除 #1，重构 #3/#4，保留 #2/#5）
- [x] LFO9.2 删除 OpenAI raw-body fallback 解析路径（#1）
- [x] LFO9.3 类型化重构：移除字符串中间 JSON 映射与解析链路（#3/#4）
- [x] LFO9.4 迁移/补齐 required-tier 测试并回填文档与日志

### LFO10 用户指令闭环复跑（2026-02-17）
- [x] LFO10.1 使用强化目标 prompt 复跑 `llm_bootstrap --ticks 30 --print-llm-io --report-json`
- [x] LFO10.2 验证“建工厂 + 制成品”达成并提取动作/错误指标
- [x] LFO10.3 记录新 TODO 并更新项目文档/devlog

### LFO11 TODO-5/TODO-6 收口（2026-02-17）
- [x] LFO11.1 `execute_until + wait` 语义收敛（改写为最小可执行动作，消除该路径 parse_error）
- [x] LFO11.2 增加结构化 `decision_rewrite` 回执并写入 trace
- [x] LFO11.3 将 `decision_rewrite` 回灌到下一轮 observation.last_action
- [x] LFO11.4 补齐 required-tier 回归测试并回填文档/devlog

### LFO9 实施结果摘要（2026-02-17）
- #1 删除（compat fallback）：
  - `OpenAiChatCompletionClient::complete` 在 `ParseBody(raw)` 分支不再走 raw-body JSON fallback，统一返回 `DecodeResponse` 错误（主请求与重试请求均一致）。
- #3/#4 删除（字符串中间链路）：
  - `LlmCompletionResult` 增加 typed turns（`LlmCompletionTurn::{Decision, ModuleCall}`）。
  - `openai_payload` 从 function_call 直接映射 typed turn，不再先序列化为字符串 JSON 再反解析。
  - `behavior_loop` 改为消费 `parse_llm_turn_payloads(turns, agent_id)`，不再依赖 `parse_llm_turn_responses(output_text, ...)`。
  - `decision_flow` 移除 JSON block 提取与文本解析函数，保留并复用决策语义解析（#2/#5）。
- 测试迁移：
  - OpenAI payload 单测迁移为 typed turn 断言；
  - `tests_part2` 解析测试改为通过 typed turn 入口验证；
  - mock client 在测试层将输出文本转换为 typed turns，仅用于测试数据构造兼容。

### LFO7 实施结果摘要（2026-02-17）
- tool 注册：
  - 在原 4 个查询工具基础上新增 `agent_submit_decision`，统一承载最终动作提交。
  - `build_responses_request_payload` 切换到 `tool_choice=required`，强制模型使用 tool call。
- 协议收敛：
  - OpenAI `function_call` 统一映射到内部解析链路；其中 `agent_submit_decision` 直接映射为决策 payload，查询工具映射为 `module_call`。
  - 移除对 `output_text` 直接决策的兼容路径（无 tool call 时返回 `EmptyChoice`），减少“文本 JSON 漂移”。
- Prompt/行为约束：
  - 系统与对话约束改为 tool-only：每轮只允许一个 tool call，禁止 JSON 文本直出。
  - 上下文足够时要求直接调用 `agent_submit_decision`，保留 `message_to_user` 参数透传。
- 回归测试：
  - 新增/更新覆盖：tool 列表数量、`tool_choice=required`、决策 tool 参数映射、`output_text` 拒绝路径、tool-only prompt 约束。

### LFO8 在线抽样结果摘要（2026-02-17）
- 运行产物：
  - `output/llm_closed_loop/tool_only_llm_bootstrap_report.json`
- 关键指标（30 tick）：
  - `action_kind_counts={build_factory:2, move_agent:1, refine_compound:3, schedule_recipe:1}`
  - `action_kind_success_counts={build_factory:1, refine_compound:3, schedule_recipe:1}`
  - `trace_counts={llm_errors:0, parse_errors:20, repair_rounds_total:20}`
  - `decision_counts={wait:23, act:7}`，`world_time=7`
- 链路确认：
  - 已完成“建工厂 + 制成品（排产）”主链路：`build_factory` 成功与 `schedule_recipe` 成功均已出现。
- 新观察 TODO（后续优化）：
  - TODO-5：tool-only 协议下，模型在排产后频繁输出 `execute_until + wait`，当前解析会判定 `execute_until action must be actionable decision` 并回落修复回路，导致长窗口 `parse_errors` 偏高、`world_time` 增长停滞。建议增加“等待语义专用协议”：
    - 方案 A：允许 `execute_until.action=wait_ticks(1)`（或等价动作）并在 guardrail 中限长；
    - 方案 B：检测到 `execute_until + wait` 时自动改写为最小可执行动作并保留 `until` 条件；
    - 方案 C：在 prompt 示例中移除 wait 型 `execute_until` 模板并增加“排产后用可执行动作推进时间”的硬约束。

### LFO10 在线复跑结果摘要（2026-02-17，用户指令复跑）
- 运行产物：
  - `output/llm_bootstrap/user_factory_closedloop_codex_2026-02-17_115631/run.log`
  - `output/llm_bootstrap/user_factory_closedloop_codex_2026-02-17_115631/report.json`
- 关键指标（30 tick）：
  - `action_success=28`、`action_failure=2`、`llm_errors=0`、`parse_errors=0`、`repair_rounds_total=0`
  - `action_kind_counts={build_factory:1, harvest_radiation:18, refine_compound:5, schedule_recipe:6}`
  - `action_kind_success_counts.schedule_recipe=4`
  - `first_action_tick={build_factory:6, schedule_recipe:8}`
  - `world_time=30`，`decision_counts={wait:0, wait_ticks:0, act:30}`
- 链路确认：
  - 已完成“建工厂 + 制成品（data）”闭环：`build_factory` 成功 1 次，`schedule_recipe` 成功 4 次。
  - `data` 从 `12` 增长至 `28`（来自 `run.log` 中 observation 采样）。
- 新观察 TODO（后续优化）：
  - TODO-6：在硬件不足但仍可恢复时，`schedule_recipe` 决策会被 guardrail 自动改写为 `refine_compound`（例如 `tick=7/9/16/24`），当前 trace 仅呈现“模型输出排产 + 执行精炼”的结果，缺少统一的“决策改写原因回执”。建议为改写链路补充结构化回执（如 `decision_rewrite: schedule_recipe -> refine_compound, reason=insufficient_resource.hardware`）并回灌到下一轮 observation，以减少策略抖动和提示词误判。

### LFO11 实施结果摘要（2026-02-17）
- 代码变更：
  - `crates/agent_world/src/simulator/llm_agent/decision_flow.rs`
  - `crates/agent_world/src/simulator/llm_agent/behavior_loop.rs`
  - `crates/agent_world/src/simulator/llm_agent.rs`
  - `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
  - `crates/agent_world/src/simulator/llm_agent/tests_part2.rs`
- TODO-5 收口：
  - `execute_until.action=wait/wait_ticks` 不再直接 parse error；解析阶段自动改写为最小可执行动作 `harvest_radiation(max_amount=1)`。
  - 线上抽样（LFO8 口径）结果：`output/llm_closed_loop/todo11_lfo8_compare_2026-02-17_121121/report.json`，`parse_errors=0`（此前样本为 20）。
- TODO-6 收口：
  - 新增结构化回执：`decision_rewrite={from,to,reason}`。
  - 回执写入 `llm_step_trace` 和 system note，并在下一轮 observation 的 `last_action.decision_rewrite` 回灌给模型。
  - 已覆盖场景：`schedule_recipe -> refine_compound` guardrail 改写、`execute_until(wait) -> harvest_radiation` 语义改写。
- 回归测试：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent --features test_tier_required -- --nocapture`
  - 新增用例：
    - `llm_parse_execute_until_rewrites_wait_action_to_minimal_harvest`
    - `llm_agent_rewrites_execute_until_wait_action_to_actionable_harvest`
  - 更新用例：
    - `llm_agent_reroutes_schedule_recipe_when_hardware_cannot_cover_one_batch`（校验 `decision_rewrite` trace + prompt 回灌）

### LFO6 实施结果摘要（2026-02-16）
- `reject_reason` 透传：`FacilityAlreadyExists` 已稳定映射为 `facility_already_exists`。
- `execute_until` 收敛：`harvest_radiation` 的 `max_ticks` 运行时硬上限为 `3`（含 auto-reentry 路径）。
- `schedule_recipe` 防过配：在 `owner=self` 且配方可识别时，`batches` 按 `self_resources.hardware` 与默认配方硬件成本上界裁剪；若单批也不可执行则回切到恢复动作（优先 `refine_compound`，否则 `harvest_radiation`），并回写 trace 提示。

### LFO6 在线复跑结果摘要（2026-02-17）
- 运行产物：
  - `output/llm_bootstrap/user_factory_closedloop_lfo6_rerun_2026-02-16_235611/run.log`
  - `output/llm_bootstrap/user_factory_closedloop_lfo6_rerun_2026-02-16_235611/report.json`
- 关键指标：
  - `action_success=27`、`action_failure=3`、`llm_errors=0`、`parse_errors=0`
  - `action_kind_counts={build_factory:1, harvest_radiation:16, move_agent:1, refine_compound:6, schedule_recipe:6}`
  - `action_kind_success_counts.schedule_recipe=4`
  - `first_action_tick={build_factory:7, schedule_recipe:9}`
- 对比 LFO5（`user_factory_closedloop_2026-02-16_230752`）：
  - 建厂与排产启动更早（`build_factory: 11 -> 7`，`schedule_recipe: 12 -> 9`）。
  - harvest 占比下降（`18 -> 16`），排产总次数上升（`5 -> 6`）。
  - 未出现 `facility_already_exists` 拒绝场景，本轮未触发该在线样本；对应 reject_reason 语义透传由单测覆盖（`llm_agent_user_prompt_preserves_facility_already_exists_reject_reason`）。
- 新观察 TODO（后续优化）：
  - TODO-4：`tick=17` 仍出现多段 JSON（含 `---` 与混合 `module_call/decision`）协议违规输出；当前依赖 guardrail/修复回路兜底（本轮 `parse_errors=0`），建议增加“多段输出硬拒绝 + 末段决策提取”策略以降低上下文膨胀和行为不确定性。

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
- 当前阶段：LFO0-LFO11 全部完成（TODO-5/TODO-6 已收口）。
- 下一步：推进 TODO-4（多段输出硬拒绝 + 末段决策提取），继续降低协议违规输出不确定性。
- 最近更新：2026-02-17（完成 LFO11 语义收敛与改写回执闭环）。
