# LLM 配置 TOML 风格统一（2026-03-02）

- 对应设计文档: `doc/world-simulator/llm/llm-config-toml-style-unification-2026-03-02.design.md`
- 对应项目管理文档: `doc/world-simulator/llm/llm-config-toml-style-unification-2026-03-02.project.md`

审计轮次: 5


## 1. Executive Summary
- 将 `config.toml` 中 LLM 配置从 `AGENT_WORLD_LLM_*` 形态统一为小写、结构化 TOML 字段。
- 保持运行行为不退化：`LlmAgentConfig` 仍可从 `config.toml` 读取完整配置，并在缺失时回退环境变量。
- 消除配置入口混杂问题，统一“文件内用 TOML 风格、环境中用 ENV 风格”的边界。

## 2. User Experience & Functionality

### In Scope
- 重构 `LlmAgentConfig::from_config_file*` 的配置读取逻辑。
- 引入并约定小写 TOML 结构：
  - `[llm]`：LLM 运行参数（超时、prompt、步数等）。
  - 根级 `profile/model_provider/model`：默认选择项。
  - `[profiles.<name>]`：模型配置档案。
  - `[model_providers.<name>]`：供应商端点与鉴权。
- 更新 `config.toml` 与 `config.example.toml`。
- 更新对应测试与文档说明。

### Out of Scope
- 不移除环境变量能力（仍作为回退与运行注入机制）。
- 不改动 `[node]` 配置结构与节点密钥自举逻辑。
- 不重构非 LLM 子系统配置加载链路。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- 新的文件内 LLM 核心字段来源：
  - `model`：`[llm].model` -> `[profiles.<selected>].model` -> 根级 `model`
  - `base_url`：`[llm].base_url` -> `[model_providers.<selected>].base_url`
  - `api_key`：`[llm].api_key` -> `[model_providers.<selected>].auth_token`
- 选择器字段：
  - 选中 profile：`[llm].profile` 或根级 `profile`
  - 选中 provider：`[llm].model_provider` 或 `[profiles.<selected>].model_provider` 或根级 `model_provider`
- 运行参数字段统一收敛到 `[llm]`：
  - `timeout_ms/system_prompt/short_term_goal/long_term_goal`
  - `max_module_calls/max_decision_steps/max_repair_rounds`
  - `prompt_max_history_items/prompt_profile`
  - `force_replan_after_same_action/harvest_max_amount_cap/execute_until_auto_reenter_ticks/debug_mode`

## 5. Risks & Roadmap
- M1：完成配置加载器重构与单测。
- M2：完成示例配置与本地配置迁移。
- M3：完成文档同步与回归验证。

### Technical Risks
- 迁移风险：旧 `config.toml` 仍使用 `AGENT_WORLD_LLM_*` 时可能读取失败。
  - 缓解：保留环境变量回退，并更新示例/文档给出新结构。
- 字段优先级风险：`llm/profile/provider` 多层回退可能产生认知偏差。
  - 缓解：在设计文档与示例中明确优先级。
- 兼容性风险：配置值类型（字符串/整数）差异导致解析失败。
  - 缓解：沿用现有 `toml_value_to_string` 转换与原有参数校验。

## 完成态（2026-03-02）
- 已完成 `LlmAgentConfig::from_config_file*` 的小写 TOML 解析重构，文件内配置统一走 `[llm]` / `[profiles.*]` / `[model_providers.*]`。
- 已迁移示例配置与文档说明，`config.example.toml` 不再使用 `AGENT_WORLD_LLM_*` 顶层键；agent 级目标覆盖改为 `[llm.agent_overrides.<AGENT_ID_NORMALIZED>]`。
- 已保留环境变量回退能力，明确“文件内 TOML 风格、环境中 ENV 风格”的边界，确保兼容运行时注入场景。
- 回归验证通过：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world simulator::llm_agent::tests::llm_config_ -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world`

## 6. Validation & Decision Record
- 追溯: 对应同名 `.project.md`，保持原文约束语义不变。
