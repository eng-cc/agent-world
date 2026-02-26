# Viewer Live 完全事件驱动改造 Phase 1（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 主循环重构：统一信号队列（先落地 Request 信号）
- [x] T2 播放脉冲线程接线：移除主循环 `recv_timeout/elapsed` 轮询
- [ ] T3 回归测试：live 关键语义不退化（play/pause/step/seek）
- [ ] T4 文档与日志收口

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：T0/T1/T2 已完成，进行 T3
- 备注：主循环已切为 `PlaybackPulse` 事件驱动 + 阻塞 `recv()`；后续在 T3/T4 收敛回归与文档。
