# Viewer Live 完全事件驱动改造 Phase 2（2026-02-26）

## 目标
- 将 `PlaybackPulse` 从“常驻固定周期发送”改为“按会话状态动态唤醒”。
- 在暂停态（以及不需要播放推进的态）不产生活动脉冲，减少空跑。
- 保持 `Play/Pause/Step/Seek` 外部协议与行为兼容。

## 范围
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`

不在范围内：
- 不引入跨模块事件总线重构。
- 不实现 LLM mailbox 全量事件化（Phase 3 处理）。

## 接口/数据
- 新增播放脉冲控制状态（内部控制器）：
  - 动态启停（enabled/disabled）
  - 状态变更唤醒（Condvar 通知）
- 脉冲线程仅在 enabled 时按 `tick_interval` 发送 `PlaybackPulse`。
- 主循环在处理 `ViewerRequest` 后同步控制状态，确保播放状态变化立即生效。

## 里程碑
1. M0：设计文档/项目管理文档建档。
2. M1：动态脉冲控制接线（暂停态零脉冲）。
3. M2：回归测试（脉冲启停语义 + live 基线语义）。
4. M3：文档收口，明确 Phase 3 入口。

## 风险
- 状态同步与线程唤醒时序可能引入竞态，需要以单一控制器为真源。
- 连接断开时若未正确唤醒等待线程，可能导致线程滞留。
