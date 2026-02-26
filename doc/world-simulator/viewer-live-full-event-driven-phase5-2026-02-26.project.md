# Viewer Live 完全事件驱动改造 Phase 5（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 共识提交事件化：NodeRuntime 等待接口 + viewer 监听信号接线
- [ ] T2 主循环背压：有界事件队列 + 事件合并/丢弃计数
- [ ] T3 回归测试：共识提交事件链路 + 背压语义
- [ ] T4 文档与日志收口

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/consensus_bridge.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/node_runtime_core.rs`
- `testing-manual.md`

## 状态
- 当前阶段：T1 已完成，进行 T2
- 备注：Phase 5 聚焦“共识提交事件化 + 有界背压”，Phase 6 再处理非共识链路去定时化。
