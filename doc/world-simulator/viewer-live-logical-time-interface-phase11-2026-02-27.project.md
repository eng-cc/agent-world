# Viewer Live 逻辑时间与事件游标接口改造 Phase 11（2026-02-27）项目管理

## 任务拆解
- [x] T0 建立 Phase 11 设计文档与项目管理文档
- [x] T1 改造 Web Test API 状态输出：新增 `logicalTime/eventSeq`，兼容保留 `tick`
- [ ] T2 改造 Web Test API 控制入口：支持 `seek_event` 并映射到现有 live seek
- [ ] T3 执行 required 测试并完成文档/devlog 收口

## 依赖
- `doc/world-simulator/viewer-live-logical-time-interface-phase11-2026-02-27.md`
- `crates/agent_world_viewer/src/web_test_api.rs`
- `crates/agent_world_viewer/src/main.rs`
- `crates/agent_world_viewer/src/main_connection.rs`
- `testing-manual.md`

## 状态
- 当前阶段：进行中（T2）
- 备注：遵循“调度事件驱动 + 内部保留逻辑时间 + 对外接口去 tick 耦合”原则。
