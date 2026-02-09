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

## 里程碑
- M1：完成 Prompt 组装抽象（结构体、模板渲染、预算裁剪）与配置读取。
- M2：完成多步状态机（plan/module/draft/finalize）与兼容分支。
- M3：完成 trace 扩展、测试补齐、README/示例配置更新与回归。

## 风险
- **步骤复杂度提升**：状态机分支增加；通过“默认直达最终决策兼容路径”降低迁移风险。
- **token 开销上升**：多步可能增加调用轮次；通过历史裁剪、上限配置与 `compact` profile 控制。
- **模型协议不稳定**：部分模型不按新类型输出；通过旧协议兼容与 repair 轮次缓冲。
- **调试成本增加**：多段 Prompt 难排查；通过 `section_trace/step_trace` 可观测字段缓解。
