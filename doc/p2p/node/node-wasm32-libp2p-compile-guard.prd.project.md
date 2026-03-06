# Agent World Runtime：Node libp2p wasm32 编译兼容守卫（项目管理文档）

审计轮次: 4
## 任务拆解（含 PRD-ID 映射）
- [x] NWC-1 (PRD-P2P-MIG-104)：设计文档与项目管理文档落地。
- [x] NWC-2 (PRD-P2P-MIG-104)：为 `agent_world_node` 增加 wasm32 目标兼容守卫与占位实现，修复编译失败。
- [x] NWC-3 (PRD-P2P-MIG-104)：稳定 `viewer::web_bridge` 重连测试并执行回归检查，回写文档/devlog。

## 依赖
- doc/p2p/node/node-wasm32-libp2p-compile-guard.prd.md
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/libp2p_replication_network.rs`
- `crates/agent_world_node/src/libp2p_replication_network_wasm.rs`
- `crates/agent_world/src/viewer/web_bridge.rs`
- `doc/scripts/precommit/pre-commit.prd.md`

## 状态
- 当前阶段：已完成（NWC-1 ~ NWC-3 全部完成）。
- 最近更新：2026-02-16。
