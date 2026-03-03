# Viewer `step` Completion Ack（2026-02-28）项目管理文档

## 任务拆解
- [x] T1 设计文档与任务拆解
- [x] T2 协议改造：控制请求增加 `request_id`，新增 completion ack 响应
- [x] T3 服务端改造：live/offline `step` 结束后回传 completion ack
- [x] T4 Viewer 改造：透传 `request_id` 并消费 completion ack 更新反馈
- [x] T5 测试与回归：proto/server/viewer targeted tests + A/B 验证
- [x] T6 文档收口：更新项目状态与 devlog

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
- 当前阶段：已完成（T1~T6）。
- 当前结论：
  - 协议/服务端/Viewer 已完成 `request_id` 贯通，`step` completion ack 可明确回传 `advanced` 或 `timeout_no_progress`。
  - 回归验证显示语义稳定性已提升：`step` 不推进时可被协议明确标记，不再仅依赖帧差分推断。
  - A/B 结果（run_id=`20260228-115832`）仍为 A PASS / B FAIL，说明 live 场景仍存在真实“超时无推进”窗口，后续可继续优化 step 成功率。
