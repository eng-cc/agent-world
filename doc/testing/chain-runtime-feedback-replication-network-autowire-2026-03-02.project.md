# Chain Runtime 反馈网络复制层自动挂载修复（2026-03-02）项目管理

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 修复 `world_chain_runtime`：启动前自动挂载默认 replication network 并启用本地无 peer fallback。
- [x] T2 测试与收口：补单测、执行 `test_tier_required` 与启动烟测、更新 devlog。

## 依赖
- `crates/agent_world/src/bin/world_chain_runtime.rs`
- `crates/agent_world/src/bin/world_chain_runtime/world_chain_runtime_tests.rs`
- `doc/testing/chain-runtime-feedback-replication-network-autowire-2026-03-02.md`
- `doc/devlog/2026-03-02.md`

## 状态
- 当前阶段：已完成（T0~T2）。
- 当前任务：无。
