# Agent World Runtime：Node libp2p wasm32 编译兼容守卫（项目管理文档）

## 任务拆解
- [x] NWC-1：设计文档与项目管理文档落地。
- [x] NWC-2：为 `agent_world_node` 增加 wasm32 目标兼容守卫与占位实现，修复编译失败。
- [x] NWC-3：稳定 `viewer::web_bridge` 重连测试并执行回归检查，回写文档/devlog。

## 依赖
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/libp2p_replication_network.rs`
- `crates/agent_world_node/src/libp2p_replication_network_wasm.rs`
- `crates/agent_world/src/viewer/web_bridge.rs`
- `doc/scripts/pre-commit.md`

## 状态
- 当前阶段：已完成（NWC-1 ~ NWC-3 全部完成）。
- 最近更新：2026-02-16。
