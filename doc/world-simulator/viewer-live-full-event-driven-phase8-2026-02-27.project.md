# Viewer Live 完全事件驱动改造 Phase 8（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 代码收敛：删除 script 回退开关与 `timer_pulse` 回退链路，仅保留 event_drive
- [ ] T2 测试改造：更新/清理回退模式断言并完成 required 回归
- [ ] T3 文档收口：更新阶段结论与遗留事项

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `crates/agent_world/src/viewer/mod.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`

## 状态
- 当前阶段：进行中（T2）
- 备注：T1 已完成，核心 live 主循环已移除 `timer_pulse` 回退链路与 script 节拍开关。
