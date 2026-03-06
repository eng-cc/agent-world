# README P2 缺口收口：Node Replication 统一到 agent_world_net 网络栈（项目管理文档）

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] NUS-0 (PRD-P2P-MIG-105)：输出设计文档与项目管理文档，并明确 wasm32 非 full node 定位。
- [x] NUS-1 (PRD-P2P-MIG-105)：根因级打破 `agent_world -> agent_world_node -> agent_world_net -> agent_world` 循环依赖，并完成 `Libp2pReplicationNetwork` 对 `agent_world_net::Libp2pNetwork` 的封装迁移。
- [x] NUS-2 (PRD-P2P-MIG-105)：补齐/调整 node replication 单测，覆盖轮换重试与无 peer 回退语义。
- [x] NUS-3 (PRD-P2P-MIG-105)：执行回归测试、更新文档与 devlog 并收口。

## 依赖
- doc/p2p/node/node-net-stack-unification-readme.prd.md
- NUS-1 依赖 NUS-0（先冻结设计边界再改实现）。
- NUS-2 依赖 NUS-1（实现稳定后再做行为回归验证）。
- NUS-3 依赖 NUS-1/NUS-2 完成。

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段：已完成（NUS-0/NUS-1/NUS-2/NUS-3 全部完成）。
- 阻塞项：无。
- 下一步：若需恢复 runtime bridge 能力，迁移到独立桥接层实现，避免重新引入包级循环依赖。
