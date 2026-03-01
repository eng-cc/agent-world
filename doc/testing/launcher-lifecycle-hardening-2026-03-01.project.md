# 启动器生命周期与就绪硬化（2026-03-01）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [ ] T1 `world_game_launcher` 修复：信号清理、启动失败回滚、就绪阶段子进程健康联动、IPv6 解析/URL 修正。
- [ ] T2 `agent_world_client_launcher` 对齐：IPv6 解析/URL 规则一致化，补单测。
- [ ] T3 回归与文档收口：执行定向测试、更新项目状态、补 devlog。

## 依赖
- `crates/agent_world/src/bin/world_game_launcher.rs`
- `crates/agent_world_client_launcher/src/main.rs`
- `doc/devlog/2026-03-01.md`

## 状态
- 当前阶段：进行中（T0 已完成，执行 T1）。
- 当前任务：T1 `world_game_launcher` 生命周期与就绪硬化。
