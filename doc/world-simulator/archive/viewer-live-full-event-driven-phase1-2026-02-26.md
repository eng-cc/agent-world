# Viewer Live 完全事件驱动改造 Phase 1（2026-02-26）

## 目标
- 将 `world_viewer_live` 主循环从 `recv_timeout` 轮询改为“统一信号队列 + 阻塞消费”的事件驱动骨架。
- 去掉基于 `last_tick.elapsed()` 的主循环内轮询判断，改为播放脉冲事件触发推进。
- 为后续“纯事件+定时唤醒器（无常驻轮询）”保留可扩展接口。

## 范围
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`

不在范围内：
- 不重写 `AgentRunner` 调度策略。
- 不重构 viewer 协议字段。
- 不在本阶段实现 mailbox/time-wheel 全量架构。

## 接口/数据
- 新增 live 循环内部信号：
  - `Request(ViewerRequest)`
  - `PlaybackPulse`
- 主循环改造为 `rx.recv()` 阻塞消费信号，不再用 `recv_timeout` 驱动轮询。
- 新增播放脉冲线程：按 `tick_interval` 发送 `PlaybackPulse`，由会话状态（`playing`）决定是否实际推进。
- 保持现有控制语义：`Play/Pause/Step/Seek` 对外协议不变。

## 里程碑
1. M0：设计文档/项目文档建档。
2. M1：主循环切换到统一信号队列。
3. M2：播放脉冲线程接线并与会话状态联动。
4. M3：回归测试与稳定性验证。

## 风险
- 线程生命周期管理不当可能导致连接退出后后台线程滞留。
- `PlaybackPulse` 与请求并发可能引入状态竞态，需要以会话状态为单一真源。
- 该阶段仍有定时脉冲线程，不等于最终“零定时轮询”终态；后续需继续 Phase 2/3 收口。

## Phase 1 完成态（2026-02-26）
- 已完成：主循环由 `recv_timeout + elapsed` 改为 `LiveLoopSignal` 队列 + 阻塞 `recv()`。
- 已完成：`PlaybackPulse` 独立线程接入，播放推进从“超时轮询”改为“事件触发”。
- 已完成：补齐 `PlaybackPulse` 线程发射/退出联动回归用例，live 测试组通过。

## 走向“完全事件驱动”的后续改造点
1. 去掉常驻固定周期脉冲线程：改成按播放状态动态启停的定时器（或 timer wheel），暂停态不产生活动事件。
2. LLM 决策改造为显式事件：把“需要决策”从布尔门控升级为 mailbox 信号（例如 `DecisionRequested/DecisionReady`），避免空调用路径。
3. 共识回放与 viewer 控制统一入同一事件总线：消除跨通道状态同步分叉，统一背压与顺序保证。
4. 引入有界队列和背压策略：防止高频事件（脉冲/外部请求）在慢消费者场景下无界堆积。
