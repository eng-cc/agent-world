# Viewer Live LLM 事件触发决策门控（2026-02-26）

- 对应设计文档: `doc/world-simulator/viewer/viewer-live-llm-event-driven-trigger-2026-02-26.design.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-llm-event-driven-trigger-2026-02-26.project.md`

审计轮次: 5

## 1. Executive Summary
- 降低 `world_viewer_live` 在 LLM 模式下的空转决策循环。
- 将 LLM 决策推进从“纯定时轮询”收敛为“有触发才推进”的事件触发门控。
- 保持现有 `play/step/chat/prompt_control` 交互能力不退化。

## 2. User Experience & Functionality
- `crates/oasis7/src/viewer/live_split_part1.rs`
- `crates/oasis7/src/viewer/live_split_part2.rs`
- `crates/oasis7/src/viewer/live/consensus_bridge.rs`
- `crates/oasis7/src/viewer/live/tests.rs`

不在范围内：
- 不重构 `WorldKernel` 的时间语义。
- 不修改 `AgentRunner` 的 `RunnerMetrics` 结构。
- 不改动 viewer 协议字段定义。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- `LiveWorld` 新增内部状态：`llm_decision_pending: bool`。
- 新增内部唤醒接口：`mark_llm_decision_pending()`。
- 决策推进规则（LLM）：
  - `llm_decision_pending=false` 时，`step()/step_via_consensus()` 直接返回空结果，不触发 LLM tick。
  - 当 `Play`、`Step`、`AgentChat` 成功、`PromptControl Apply/Rollback` 成功、或共识提交事件应用后，重新置 `llm_decision_pending=true`。
  - 一次 LLM 决策若未产出 action/event，则回落到 `llm_decision_pending=false`，等待下一次外部触发。

## 5. Risks & Roadmap
1. M1：补齐 `LiveWorld` 决策门控状态与普通 live 路径接线。
2. M2：补齐 consensus 路径门控与提交后唤醒。
3. M3：补齐请求入口唤醒（play/step/chat/prompt apply rollback）。
4. M4：新增回归测试验证“避免重复空 tick”。

### Technical Risks
- `Step {count > 1}` 在“无新触发”场景下有效步数可能小于 `count`（符合事件触发语义，但与旧轮询语义有体感差异）。
- `Preview` 仍不触发自动决策（设计上仅观测，不改变世界状态）。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.project.md`，保持原文约束语义不变。
