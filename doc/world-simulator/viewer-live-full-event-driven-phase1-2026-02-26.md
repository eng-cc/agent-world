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
