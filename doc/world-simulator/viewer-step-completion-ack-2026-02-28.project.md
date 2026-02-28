# Viewer `step` Completion Ack（2026-02-28）项目管理文档

## 任务拆解
- [x] T1 设计文档与任务拆解
- [ ] T2 协议改造：控制请求增加 `request_id`，新增 completion ack 响应
- [ ] T3 服务端改造：live/offline `step` 结束后回传 completion ack
- [ ] T4 Viewer 改造：透传 `request_id` 并消费 completion ack 更新反馈
- [ ] T5 测试与回归：proto/server/viewer targeted tests + A/B 验证
- [ ] T6 文档收口：更新项目状态与 devlog

## 依赖
- `crates/agent_world_proto/src/viewer.rs`
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/server.rs`
- `crates/agent_world_viewer/src/main_connection.rs`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `testing-manual.md`
- `scripts/run-game-test-ab.sh`

## 状态
- 当前阶段：进行中（T1 完成，T2~T6 待完成）。
- 当前结论：
  - 需求边界已明确：`step` completion ack 必须按 `request_id` 关联。
  - 实现策略采用“协议增强 + viewer 降级兼容”双轨，避免新旧版本互操作退化。
