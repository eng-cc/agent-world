# Agent World Simulator：LLM Prompt 组装重构与多步决策机制（设计文档）

## 目标
- 解决当前 `system_prompt + user_prompt` 直接拼接过于生硬的问题，形成可维护、可扩展、可裁剪的 Prompt 组装层。
- 在现有 `module_call -> decision` 基础上，补齐“计划-检索-草案-定稿”多步机制，降低一次性输出失败率。
- 保持 simulator 稳定性：任一阶段超限、解析失败或模块调用异常时，均可安全降级为 `Wait`。
- 为后续 runtime 审计链路与 viewer 诊断面板提供更细粒度的步骤级可观测数据。

## 范围

### In Scope
- 重构 Prompt 组装：从字符串硬编码改为分段结构化组装（角色、目标、上下文、协议、示例、历史）。
- 引入多步决策状态机：`plan -> module_call* -> decision_draft -> final_decision`。
- 扩展配置项：步骤上限、草案校验轮次、Prompt 预算策略。
- 扩展 trace：记录每步输入摘要、输出类型、校验结果、降级原因。
- 补充单元测试：Prompt 组装、步骤流转、上限保护、兼容旧协议。

### Out of Scope
- 引入外部 function calling/tool calling SDK（本期仍使用 JSON 协议）。
- 引入向量检索、RAG、长期记忆重评分。
- 引入多模型路由/仲裁或成本预算优化。
- 将完整 prompt 分片持久化到 runtime 分布式存储。

## 现状问题（补充说明）
- **拼接耦合高**：`llm_agent.rs` 中 `system_prompt()` 与 `user_prompt()`直接拼接文本，角色规则、目标与上下文难以独立演进。
- **上下文缺少分层预算**：观测与模块历史直接序列化，缺少按优先级裁剪，容易出现 token 膨胀。
- **步骤语义不足**：当前只区分 `module_call` 和最终 `decision`，缺少显式计划阶段与决策草案校验阶段。
- **恢复策略粗糙**：解析失败主要直接降级，缺少“格式修复/重述”这一轻量恢复路径。

## 接口 / 数据

### 1) Prompt 组装接口（建议）
- 新增结构：
  - `PromptAssemblyInput`
    - `agent_id`
    - `observation`
    - `module_history`
    - `memory_digest`
    - `step_context`（当前阶段、已用步数、剩余额度）
  - `PromptSection`
    - `kind`（`policy/goals/context/tools/history/output_schema/examples`）
    - `priority`（高/中/低）
    - `content`
    - `token_budget`（可选）
  - `PromptAssemblyOutput`
    - `system_prompt`
    - `user_prompt`
    - `section_trace`（每段是否被裁剪）
- 组装规则：
  - 固定先后顺序：`policy -> goals -> context -> tools -> history -> output_schema`。
  - 当预算不足时，从低优先级段开始裁剪，保留协议与动作白名单段。
  - 模块历史默认仅保留最近 `N` 条 + 摘要，不再每轮全量重复。

### 2) 新增配置项
- `AGENT_WORLD_LLM_MAX_DECISION_STEPS`：单次 `decide` 最大步骤数（默认 `4`）。
- `AGENT_WORLD_LLM_MAX_REPAIR_ROUNDS`：解析失败后的格式修复轮次（默认 `1`）。
- `AGENT_WORLD_LLM_PROMPT_MAX_HISTORY_ITEMS`：Prompt 中注入的模块历史条数上限（默认 `4`）。
- `AGENT_WORLD_LLM_PROMPT_PROFILE`：Prompt 模板档位（`compact`/`balanced`，默认 `balanced`）。

### 3) 多步协议（JSON）
- 规划步骤：
  - `{"type":"plan","missing":["memory","observation_detail"],"next":"module_call"}`
- 模块调用（兼容现有）：
  - `{"type":"module_call","module":"memory.short_term.recent","args":{"limit":5}}`
- 决策草案：
  - `{"type":"decision_draft","decision":{"decision":"move_agent","to":"loc-2"},"confidence":0.72,"need_verify":true}`
- 最终决策（兼容现有）：
  - `{"decision":"wait"}`
  - `{"decision":"wait_ticks","ticks":3}`
  - `{"decision":"move_agent","to":"loc-2"}`
  - `{"decision":"harvest_radiation","max_amount":20}`

说明：为保持兼容，若模型直接输出最终 `decision`，可跳过 `plan/decision_draft`。

### 4) 决策状态机（建议）
1. `StepPlan`：首轮要求输出 `plan` 或直接最终 `decision`。
2. `StepModuleLoop`：按计划执行 `module_call`，最多 `max_module_calls` 次。
3. `StepDecisionDraft`：要求输出 `decision_draft`，并做字段校验。
4. `StepFinalize`：输出最终 `decision`；若失败进入 `repair`（受 `max_repair_rounds` 限制）。
5. 任一阶段超限或不可恢复错误 -> `Wait` 并记录 `parse_error/degrade_reason`。

### 5) Trace 扩展（`AgentDecisionTrace`）
- 新增可选字段（示意）：
  - `llm_step_trace`: `Vec<LlmStepTrace>`
    - `step_index`
    - `step_type`（plan/module_call/decision_draft/final_decision/repair）
    - `input_summary`
    - `output_summary`
    - `status`（ok/error/degraded）
  - `llm_prompt_section_trace`: `Vec<LlmPromptSectionTrace>`（段级裁剪和预算命中信息）


## 上下文长度与记忆平衡策略（补充）

### 设计原则
- **预算先行**：先算可用输入预算，再决定注入多少观测/记忆，避免“先拼接再截断”。
- **记忆不等于上下文**：`AgentMemory` 是候选池，Prompt 只注入“当前决策必需”的最小证据集。
- **阶段化注入**：按 `plan/module_call/decision_draft/final_decision` 阶段动态分配上下文预算。
- **可观测可调参**：每轮记录“候选数/入选数/裁剪比例/估算 token”，支持在线调参。

### 预算模型
- 定义：
  - `context_window`：模型上下文窗口（来自 profile/model 配置）。
  - `reserved_output_tokens`：为模型输出预留 token（按阶段动态设置）。
  - `safety_margin_tokens`：安全边际（建议固定 10% 或最小 512）。
  - `effective_input_budget = context_window - reserved_output_tokens - safety_margin_tokens`。
- 预算分桶（默认 `balanced`）：
  - `policy + output_schema`：硬保留，不参与裁剪。
  - `observation_core`：硬保留（关键状态字段）。
  - `memory_selected`：弹性区（可裁剪）。
  - `module_history`：低优先级弹性区（优先裁剪）。
  - `examples`：最低优先级（预算紧张时先移除）。

### 记忆选择策略
- 候选池：
  - 短期记忆：最近 `N_st` 条（默认 12）。
  - 长期记忆：检索 `N_lt` 条（默认 20，按 query 或重要度召回）。
- 打分函数（建议）：
  - `score = 0.45*relevance + 0.25*recency + 0.20*importance + 0.10*failure_bonus`。
- 过滤与去重：
  - 同类重复事件做语义归并（相同 action/reason 合并为摘要）。
  - 相同 location/agent 的连续观测仅保留最新一条 + 统计摘要。
- 打包策略：
  - 先放高分短句摘要，再在有预算时补充原始条目。
  - 默认目标：短期入选 `K_st=4`、长期入选 `K_lt=6`。

### 分阶段预算分配（建议默认）
- `StepPlan`：强调目标、观测核心、记忆摘要；不注入详细模块历史。
- `StepModuleLoop`：仅注入当前模块请求所需最小上下文 + 最近 1~2 次模块结果。
- `StepDecisionDraft`：强化动作约束、失败案例记忆、资源边界。
- `StepFinalize`：仅保留决策 schema、关键证据、最终校验提示。

### 裁剪顺序与降级策略
1. 移除 `examples`。
2. 压缩 `module_history`（保留最近 M 条）。
3. 长期记忆从低分开始裁剪。
4. 短期记忆从低分开始裁剪。
5. 观测降维为核心字段（位置、资源、电力、热状态、可见目标）。
6. 若仍超预算，切换 `compact` profile；再失败则降级 `Wait` 并记录 `degrade_reason=prompt_budget_exceeded`。

### 可观测性与验收
- 新增建议指标：
  - `llm_prompt_estimated_tokens`、`llm_prompt_budget_used_ratio`
  - `llm_memory_candidates_total`、`llm_memory_selected_total`
  - `llm_prompt_truncation_count`、`llm_prompt_profile_switch_count`
- 验收基线（建议）：
  - 在固定场景下运行 1000 tick，不出现因超长上下文导致的连续解析失败风暴。
  - 平均 `budget_used_ratio` 控制在 0.75~0.9。
  - memory 注入条目中，高相关命中率（由诊断标签统计）高于 70%。


## 反重复门控与持续执行动作（LMSO14 补充）

### 目标
- 解决真实长跑中“模型持续输出同一动作”的复读风险。
- 在强制再规划的同时，保留“确实需要连续执行同一动作”的表达能力。

### 新增配置
- `AGENT_WORLD_LLM_FORCE_REPLAN_AFTER_SAME_ACTION`：连续同动作阈值（默认 `4`，`0` 表示关闭）。
  - 当连续动作达到阈值时，下一轮 `decide` 进入反重复门控，优先要求 `plan/module_call`。

### Prompt 策略补充
- 在 `Goals/Tool Protocol` 中加入反停滞与探索偏置：
  - 无新证据时避免重复动作。
  - 局部状态长期不变时优先探索。
- 在门控触发时动态注入 `[Anti-Repetition Guard]`：
  - 先补证据（`plan/module_call`），再输出最终决策。
  - 若确需连续动作，要求输出 `execute_until`。

### 新增决策协议
- `execute_until`（终态决策的一种）：
  - 单事件：`{"decision":"execute_until","action":{<decision_json>},"until":{"event":"action_rejected"},"max_ticks":<u64>}`
  - 多事件（任一命中即停止）：`{"decision":"execute_until","action":{<decision_json>},"until":{"event_any_of":["action_rejected","new_visible_agent"]},"max_ticks":<u64>}`
  - 兼容写法：`until.event` 支持 `"a|b"`（按多事件解析）
- 语义：
  - `action`：需要重复执行的动作（当前支持 `move_agent` / `harvest_radiation`）。
  - `until.event` / `until.event_any_of`：停止条件。
  - `max_ticks`：硬上限，避免无限循环。

### 运行时行为
- 当存在激活中的 `execute_until` 计划且停止条件未满足时：
  - 跳过当轮 LLM 请求，直接复用计划动作执行。
  - 记录 `execute_until_continue` trace，保持可观测性。
- 停止条件命中后：
  - 清理持续计划并恢复正常 LLM 决策流。

### 协议容错（LMSO15）
- `until.event` 除单值外，兼容 `"a|b"` 与 `"a,b"`，按“任一事件命中即停止”解释。
- 新增 `until.event_any_of`（数组）作为首选多事件表达。
- `decision_draft` 兼容简写：当 `decision` 为字符串时，可直接复用同层的 `to/max_amount/ticks` 字段。
- LLM 输出解析改为优先提取“首个完整 JSON 对象/数组”，降低多段输出导致的 trailing-chars 失败率。

### 运行日志体积治理（LMSO16）
- `world_llm_agent_demo` 新增 `--llm-io-max-chars <n>`：对每 tick 的 `llm_input/llm_output` 打印做字符截断。
- 截断策略：保留前 `n` 字符，并追加 `...(truncated, total_chars=..., max_chars=...)` 标记，便于审计与限流并存。
- `scripts/llm-longrun-stress.sh` 新增同名参数并透传；`summary.txt` 追加 `llm_io_max_chars` 字段，便于跨版本对齐。

### LLM 超时策略（LMSO17）
- 将默认 `AGENT_WORLD_LLM_TIMEOUT_MS` 提升到 `180000`（3 分钟），适配真实运行态中“几分钟响应”的模型调用。
- 保留短超时自动回退机制：当显式配置小于默认值时，首次用短超时请求，超时后自动用默认超时重试一次。
- 目标：减少正常慢响应被误判为错误，同时保留低延迟偏好下的快速失败能力。

### until.event 扩展语义（LMSO18）
- 在 `execute_until` 中新增停止事件：`insufficient_electricity`、`thermal_overload`、`harvest_yield_below`、`harvest_available_below`。
- 阈值事件约束：
  - `harvest_yield_below` / `harvest_available_below` 必须提供 `until.value_lte`。
  - `until.value_lte` 必须为非负整数。
- 运行态语义绑定：
  - `insufficient_electricity`：上一轮动作被拒绝，且拒绝原因为电力不足（`InsufficientResource(Electricity)`）或 `AgentShutdown`。
  - `thermal_overload`：上一轮动作被拒绝，且拒绝原因为 `ThermalOverload`。
  - `harvest_yield_below`：上一轮 `RadiationHarvested.amount <= until.value_lte`。
  - `harvest_available_below`：上一轮 `RadiationHarvested.available <= until.value_lte`。
- 协议兼容：保留 `until.event_any_of` 与 `until.event` 的 `"a|b"` / `"a,b"` 多事件写法，按“任一命中即停止”解释。

### 上下文收敛与压测可观测性（LMSO19）
- 问题：真实运行态中，`module_call` 返回大 payload（尤其记忆检索）会放大后续 Prompt 中 `Module History` 段，导致输入峰值抖动。
- 方案：Prompt 注入前对 `ModuleCallExchange.result` 做软压缩：
  - 小结果保持原结构。
  - 大结果改写为 `{truncated, original_chars, preview}`，`preview` 使用字符级截断。
- 预期：不影响模块可用性前提下，降低单轮极端 Prompt 峰值，减少 section 裁剪触发。
- 压测脚本补强：`scripts/llm-longrun-stress.sh` 在无 `jq` 环境下改为 Python 解析 `report.json`，保证 `prompt_section_clipped` 等指标准确落盘。

### OpenAI 兼容 Tool 注册与解析（LMSO20）
### 单轮多段输出顺序消费（LMSO21）
- 问题：真实长跑中，模型可能在一次回复里输出多段 JSON（例如连续 `module_call` 后再给 `decision_draft/final decision`），当前只消费首段会导致：
  - 后续段被丢弃，额外增加回合与输入体积。
  - 在模块调用上限附近更容易触发 `module call limit exceeded` 与 `Wait` 降级。
- 方案：
  - 在每次 LLM completion 后提取并顺序解析所有 JSON 块（支持 `---`、空行或自由文本夹杂）。
  - 同一轮内按顺序执行状态迁移：`plan/module_call/decision_draft/final_decision/execute_until`。
  - 若同轮先出现若干 `module_call`，后续出现 `decision_draft/final decision`，则继续消费直到产出终态决策或耗尽片段。
- 兼容策略：
  - 保留旧行为兼容：仅单段 JSON 的输出路径不变。
  - 对超过 `max_module_calls` 的额外 `module_call` 采用“软拒绝并继续消费同轮后续片段”，避免过早终止整轮。
- 预期：
  - 降低 `no terminal decision` 与 `module call limit exceeded` 的连锁概率。
  - 在不增加 LLM 请求次数的前提下提升多步协议收敛性。

- 问题：当前模块调用主要依赖 `{"type":"module_call",...}` 文本协议，未充分利用 OpenAI `tools/tool_calls` 原生结构，跨模型兼容性与结构化约束不足。
- 方案：将模块能力以 OpenAI `tools` 形态注册到请求层（现已迁移至 `async-openai` Responses API），并在响应侧优先解析函数调用：
  - 请求注册：每轮请求携带函数工具定义（name/description/parameters），`tool_choice=auto`。
  - 名称兼容：对外使用 OpenAI 友好命名（`agent_modules_list` 等），内部映射回既有模块名（`agent.modules.list` 等）。
  - 响应解析优先级：`responses.output.function_call` > 旧 `type=module_call` 文本 JSON > 其他决策协议。
  - 回退策略：保留旧协议兼容，不强制要求所有模型必须走 tool_call。
- 预期：
  - 减少“自然语言夹带 JSON”导致的解析抖动。
  - 让模块调用语义更贴近 OpenAI 标准接口，便于后续接入更多兼容端。

### 风险与约束
- 风险：过强门控可能打断合理的重复动作。
- 缓解：通过 `execute_until` 显式表达“重复直到事件”，并允许阈值配置化关闭门控。


## 里程碑
- M1：完成 Prompt 组装抽象（结构体、模板渲染、预算裁剪）与配置读取。
- M2：完成多步状态机（plan/module/draft/finalize）与兼容分支。
- M3：完成 trace 扩展、测试补齐、README/示例配置更新与回归。

## 风险
- **步骤复杂度提升**：状态机分支增加；通过“默认直达最终决策兼容路径”降低迁移风险。
- **token 开销上升**：多步可能增加调用轮次；通过历史裁剪、上限配置与 `compact` profile 控制。
- **模型协议不稳定**：部分模型不按新类型输出；通过旧协议兼容与 repair 轮次缓冲。
- **调试成本增加**：多段 Prompt 难排查；通过 `section_trace/step_trace` 可观测字段缓解。

## 场景测试观察与后续优化点（2026-02-10）

### 测试样本
- 场景：`llm_bootstrap`
- 运行：`--ticks 30`
- 报告：`.tmp/llm_multi_round_30/report.json`

### 观察结论
- 稳定性：`llm_errors=0`，说明迁移到 Responses API 后主链路可持续运行。
- 解析质量：`parse_errors=4`、`repair_rounds_total=3`，说明输出格式兼容仍需增强。
- 效率：`total_actions=26/30`、`wait=4`，存在一定空转。
- 上下文：`llm_input_chars_max=18175` 且 `prompt_section_clipped=8`，提示上下文预算压力仍在。

### 后续优化点（建议纳入下一轮任务）
1. 扩展多段输出解析兼容（包含文本夹杂 JSON、字段缺省与顺序漂移）。
2. 增加动作参数安全护栏（尤其是 `harvest_radiation.max_amount` 上限控制）。
3. 优化 `execute_until` 的事件模板与重入策略，进一步降低 `Wait` 占比。
4. 继续收敛 Prompt 体积：加大 history/memory 摘要折叠与优先级裁剪力度。


## LMSO23 输入侧收敛优化（2026-02-10）

### 触发问题（输入侧）
- 部分实跑日志出现“单轮输出多个 JSON 块 + 连续 module_call”模式，导致 `no terminal decision` 或 `module call limit exceeded`。
- 典型样本见：
  - `.tmp/llm_stress_lmso21_2tick_smoke/run.log`
  - `.tmp/llm_stress_lmso20_10_closure/run.log`

### 已实施优化
1. Prompt 工具协议增加硬约束：
   - 每轮仅允许一个 JSON 对象（禁止 `---` 多段与代码块包裹）。
   - 每轮若输出 `module_call`，只允许一个，且不与 `decision*` 混合。
   - 显式模块白名单（4 个内置模块），抑制幻觉模块名。
2. Step 元信息增加收敛指标：
   - `module_calls_remaining` / `turns_remaining` / `must_finalize_hint`。
3. 输出 Schema 增加强约束：
   - `decision_draft.decision` 必须是完整对象（非字符串）。
   - `execute_until` 仅允许作为最终 `decision` 输出。
4. Step orchestration 增加动态门槛：
   - 当 `module_calls_remaining <= 1` 或 `turns_remaining <= 1`，强制本轮输出最终 `decision`。

### 验证结果
- 历史对照（同类 smoke 场景）：`.tmp/llm_stress_lmso21_2tick_smoke/report.json` 为 `parse_errors=1`。
- 优化后 smoke 复测：
  - `.tmp/lmso23_prompt_2_smoke/report.json`：`parse_errors=0`，`llm_errors=0`。
  - `.tmp/lmso23_prompt_4/report.json`：`parse_errors=0`，`llm_errors=0`。
- 优化后 30 tick 全量回归：
  - 基线（优化前）`.tmp/llm_multi_round_30/report.json`：`parse_errors=4`、`repair_rounds_total=3`、`decision.wait=4`、`prompt_section_clipped=8`。
  - 当前（优化后）`.tmp/lmso23_prompt_30_final/report.json`：`parse_errors=0`、`repair_rounds_total=0`、`decision.wait=0`、`prompt_section_clipped=0`。
  - 附加收益：`llm_input_chars_avg` 从 `3205` 降到 `1542`，`llm_input_chars_max` 从 `18175` 降到 `14056`。

### 后续
- LMSO24 将处理动作参数护栏（如 `harvest_radiation.max_amount` 上限）与更长轮次稳定性验证。
