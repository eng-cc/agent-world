# Agent World Simulator：LMSO29 可用性与稳定性收敛（项目管理文档）

## 任务拆解
- [x] LMSO29A 放宽 Prompt peak 预算阈值（soft/hard reserve 与 soft cap）。
- [x] LMSO29B 放宽 Prompt 素材压缩上限（history/memory/observation）。
- [x] LMSO29C 30 tick 回归对比并验收链路可用性指标。
- [x] LMSO29D 文档回写与开发日志更新。

## 依赖
- `crates/agent_world/src/simulator/llm_agent.rs`
- `crates/agent_world/src/simulator/llm_agent/prompt_assembly.rs`
- `crates/agent_world/src/simulator/llm_agent/tests.rs`
- `doc/world-simulator/llm-lmso29-stability.md`

## 状态
- 当前阶段：LMSO29（首轮收口）
- 目标：在不破坏链路稳定性的前提下，适度放宽 token 限制并回收策略表现。
- 最近更新：完成 LMSO29 首轮实现与 30 tick 回归（2026-02-11）。

## 回归结论（30 tick）
- 基线：`.tmp/lmso28_peak_30_final/report.json`
- 当前：`.tmp/lmso29_final_30_promptfix/report.json`（并补充对照 `.tmp/lmso29_final_30_rerun/report.json`、`.tmp/lmso29_tuned_30/report.json`）
- 链路可用性：`llm_errors=0`，`prompt_section_clipped=0`；存在少量 `parse_errors/repair_rounds` 波动。
- 指标观察：
  - `llm_input_chars_total/avg/max` 相比 LMSO28 出现回升（放宽 token 与策略波动叠加）。
  - `action_success` 未回收到 LMSO28 水位，当前仍需继续优化。
- 后续：进入 LMSO29.1，重点是“动作失败回收优先、输入体积次优先”，并保持 `llm_errors=0`。
