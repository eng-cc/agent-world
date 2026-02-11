# Agent World Simulator：LMSO29 可用性与稳定性收敛（设计文档）

## 目标
- 在保持链路可用（`llm_errors=0`、`parse_errors=0`）前提下，回收 LMSO28 后的策略稳定性波动。
- 允许适度放宽 token 预算限制，减少过度压缩导致的动作决策信息不足。
- 保持多轮决策链路（plan/module_call/decision/execute_until）兼容，不引入协议破坏性变更。

## 范围
- In Scope：
  - 放宽 Prompt 预算收敛阈值（soft/hard reserve 与 soft section cap）。
  - 放宽 Prompt 内容压缩上限（module history args/result、memory digest、observation 注入规模）。
  - 基于 `llm_bootstrap` 场景执行 30 tick 回归，验证链路可用性与稳定性指标。
- Out of Scope：
  - 模型与 provider 切换。
  - 决策协议字段新增/删减（JSON schema 不变）。
  - world kernel 业务规则改造。

## 接口 / 数据
- 主要改动接口：
  - `LlmPromptProfile::prompt_budget()`：调整输入预算有效空间。
  - `PromptAssembler::peak_targets_tokens()`：调整 peak soft/hard 目标。
  - `PromptAssembler::apply_peak_budget()`：调整软段压缩上限。
  - Prompt 素材压缩常量：`PROMPT_MODULE_*`、`PROMPT_MEMORY_*`、`PROMPT_OBSERVATION_*`。
- 验证数据：
  - `.tmp/<run>/report.json`（核心指标对比）。
  - `.tmp/<run>/run.log`（逐 tick 现象与异常定位）。

## 里程碑
- M1：完成预算/压缩阈值放宽改造与单元测试。
- M2：完成 30 tick 回归并对比 LMSO28 基线，确认链路稳定性不退化。
- M3：更新项目文档、devlog 并提交。

## 风险
- **放宽过度风险**：输入体积反弹导致峰值再次失控；通过 30 tick 指标阈值与对比回归控制。
- **样本波动风险**：单次 run 的动作成功率存在随机波动；通过固定场景同口径对比，先验证链路稳定再逐步扩样。
- **兼容风险**：放宽预算后 prompt 结构变化可能触发边缘解析分支；通过现有 parser/repair 单测回归兜底。

## 已实施与验证（2026-02-11）
- 已实施：
  - 适度放宽 `PromptBudget` 有效输入空间（降低 safety margin；保留输出预留不变）。
  - 新增并校验 peak/soft cap 相关单测，确保预算逻辑可回归。
  - 在 prompt 约束中增加 `move_agent.to` 不得指向当前位置的显式规则，降低无效移动概率。
- 已验证（命令）：
  - `env -u RUSTC_WRAPPER cargo test -p agent_world prompt_budget_peak_targets_use_relaxed_reserve_values -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_prompt_profile_uses_relaxed_token_budget_for_stability -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo test -p agent_world llm_agent_prompt_contains_execute_until_and_exploration_guidance -- --nocapture`
  - `env -u RUSTC_WRAPPER cargo check -p agent_world`
  - `env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_llm_agent_demo -- llm_bootstrap --ticks 30 --report-json .tmp/lmso29_final_30_promptfix/report.json`

## 回归观察
- 对比 LMSO28 基线（`.tmp/lmso28_peak_30_final/report.json`）：
  - 稳定性主链保持：`llm_errors=0`。
  - 存在轻度波动：`parse_errors` 与 `repair_rounds_total` 偶发非零。
  - `action_success` 暂未回收至 LMSO28 水位，且输入体积指标有回升，说明“放宽 token + 当前策略”仍需二次收敛。

## 后续优化点（LMSO29.1）
- 先收敛动作失败（特别是 move 失败模式）再扩大 token 放宽范围。
- 引入“多次 30 tick + 固定统计口径”的验收方式，降低单次采样偏差影响。
- 在保持 `llm_errors=0` 的前提下，逐步回收 `llm_input_chars_max` 与 `module_call` 波动。
