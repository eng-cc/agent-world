# oasis7 Runtime：共识代码统一收敛到 agent_world_consensus（项目管理文档）

- 对应设计文档: `doc/p2p/consensus/consensus-code-consolidation-to-agent-world-consensus.design.md`
- 对应需求文档: `doc/p2p/consensus/consensus-code-consolidation-to-agent-world-consensus.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] CCG-0 (PRD-P2P-MIG-058)：完成设计文档与项目管理文档。
- [x] CCG-1 (PRD-P2P-MIG-058)：在 `agent_world_consensus` 实现可复用 node_pos 核心状态机，并在 `agent_world_node` 完成接线替换。
- [x] CCG-2 (PRD-P2P-MIG-058)：补齐/回归测试并回写文档与 devlog。
- [x] CCG-3 (PRD-P2P-MIG-058)：扩展设计/项目文档，定义第二阶段全收口任务。
- [x] CCG-4 (PRD-P2P-MIG-058)：将 `agent_world_node` 残留共识纯逻辑（action/signature/message）迁移到 `agent_world_consensus` 并接线。
- [x] CCG-5 (PRD-P2P-MIG-058)：完成 PoS 单链路收敛（`pos` 复用 `node_pos` 推进核心）、定向回归和文档/devlog 收口。

## 依赖
- `doc/p2p/consensus/consensus-code-consolidation-to-agent-world-consensus.prd.md`
- `crates/agent_world_consensus/src/lib.rs`
- `crates/agent_world_consensus/src/node_consensus_action.rs`
- `crates/agent_world_consensus/src/node_consensus_message.rs`
- `crates/agent_world_consensus/src/node_consensus_signature.rs`
- `crates/agent_world_consensus/src/dht.rs`
- `crates/agent_world_consensus/src/network.rs`
- `crates/agent_world_consensus/src/node_pos.rs`
- `crates/agent_world_consensus/src/pos.rs`
- `crates/agent_world_node/src/lib.rs`
- `crates/agent_world_node/src/gossip_udp.rs`
- `crates/agent_world_node/src/node_runtime_core.rs`

## 状态
- 当前阶段：CCG-0~CCG-5 全部完成。
- 阻塞项：无。
- 最近更新：2026-02-20。
