# Viewer Live LLM 事件触发决策门控（项目管理）

## 任务拆解
- [x] T0 建档：设计文档与项目管理文档
- [x] T1 `LiveWorld` 增加 LLM 决策门控状态，并接入普通 live `step()`
- [x] T2 consensus 路径接入门控与提交后唤醒
- [x] T3 请求入口接入唤醒：`Play/Step/AgentChat/PromptControl Apply/Rollback`
- [x] T4 回归测试：验证空结果下不会重复累加空决策 tick

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/consensus_bridge.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：已完成（T0~T4）
- 验证结果：`live_world_llm_event_driven_gate_avoids_repeated_empty_ticks` 通过
- 后续可选项：若需要进一步降载，可在播放循环层引入“长期空结果自动降频/暂停”策略
