# Viewer Live LLM 事件触发门控设计

- 对应需求文档: `doc/world-simulator/viewer/viewer-live-llm-event-driven-trigger-2026-02-26.prd.md`
- 对应项目管理文档: `doc/world-simulator/viewer/viewer-live-llm-event-driven-trigger-2026-02-26.project.md`

## 1. 设计定位
定义 LLM live 决策的事件触发门控方案：只有在外部触发存在时才推进 LLM 决策，避免空转 tick 持续消耗。

## 2. 设计结构
- 门控状态层：`LiveWorld.llm_decision_pending` 表示是否允许下一次 LLM 决策推进。
- 触发唤醒层：`Play/Step/AgentChat/PromptControl` 成功及共识提交后重新置 pending。
- 空结果降载层：一次 LLM 决策若未产出 action/event，则自动回落为 `false`。
- 普通/共识双路径层：普通 live 与 consensus bridge 共享同一门控语义。

## 3. 关键接口 / 入口
- `llm_decision_pending`
- `mark_llm_decision_pending()`
- `live_split_part1.rs` / `live_split_part2.rs`
- `live/consensus_bridge.rs`

## 4. 约束与边界
- 不重构 `WorldKernel` 时间语义，也不修改 `RunnerMetrics` 结构。
- Preview 继续只承担观测，不触发自动决策。
- Step 多步请求在无新触发时可提前停止，属于事件驱动语义的一部分。
- 门控逻辑必须覆盖普通 live 与共识路径，避免一侧重新出现空转。

## 5. 设计演进计划
- 先补门控状态与普通 live 接线。
- 再接 consensus 提交后唤醒。
- 最后通过回归测试验证“无触发不空转”。
