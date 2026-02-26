# Viewer Live 完全事件驱动改造 Phase 5（2026-02-26）

## 目标
- 将共识提交消费从“`PlaybackPulse` 周期拉取”改造为“提交到达触发主循环信号”的事件驱动链路。
- 引入 live 主循环有界事件队列与过载统计，避免高压场景无界积压。
- 保持现有 viewer 对外协议兼容（请求/响应字段不变）。

## 范围
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/consensus_bridge.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/node_runtime_core.rs`

不在范围内：
- 不改动 viewer websocket/http 协议。
- 不做跨进程事件总线（仅限当前进程内事件化）。

## 接口/数据
- NodeRuntime 新增“等待提交批次”接口（阻塞等待/超时返回），供 viewer 监听线程消费。
- live 主循环新增共识提交信号：`ConsensusCommitted`（或等价内部信号），提交到达即入队处理。
- live 事件队列改为有界容量：
  - 达到上限后对可合并事件执行合并（例如重复 LLM 唤醒/播放脉冲）。
  - 记录丢弃/合并计数，供日志与排障使用。

## 里程碑
1. M0：设计文档/项目文档建档。
2. M1：NodeRuntime 提交批次等待接口 + viewer 共识监听线程接线。
3. M2：主循环有界事件队列与基础背压策略（合并/计数）。
4. M3：回归测试（共识提交事件触发 + 队列背压语义）。
5. M4：文档收口并明确下一阶段（非共识路径进一步去定时化）。

## 风险
- NodeRuntime 增加等待接口后若通知丢失，可能导致 viewer 卡住或提交延迟可见。
- 有界队列与事件合并策略若设计不当，可能造成控制命令饿死或响应顺序漂移。
