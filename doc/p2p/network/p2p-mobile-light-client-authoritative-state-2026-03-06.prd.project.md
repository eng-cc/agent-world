# P2P Mobile Light Client 权威状态架构（项目管理文档）

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] TASK-P2P-MLC-001 (PRD-P2P-MLC-001/002/003/004) [test_tier_required]: 输出专题 PRD、项目管理文档，并回写模块主 PRD/索引链路。
- [x] TASK-P2P-MLC-002 (PRD-P2P-MLC-001) [test_tier_required]: 实现移动端 intent-only 协议字段（`tick/seq/sig`）与网关去重/幂等 ACK。
- [ ] TASK-P2P-MLC-003 (PRD-P2P-MLC-002) [test_tier_required]: 实现权威批次提交（`state_root/data_root`）和客户端 `pending/confirmed/final` 状态机。
- [ ] TASK-P2P-MLC-004 (PRD-P2P-MLC-002/003) [test_tier_full]: 实现 challenge/resolve/slash 链路与 watcher 复算入口。
- [ ] TASK-P2P-MLC-005 (PRD-P2P-MLC-004) [test_tier_required]: 实现链重组回滚、断线重连快照追平、会话吊销换钥流程。
- [ ] TASK-P2P-MLC-006 (PRD-P2P-MLC-001/002/003/004) [test_tier_full]: 执行 required/full 联合回归并沉淀发布门禁证据。

## 依赖
- `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.md`
- `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.project.md`
- `doc/p2p/prd.md`
- `doc/p2p/network/readme-p1-network-production-hardening.prd.md`
- `doc/p2p/distributed/distributed-runtime.prd.md`
- `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md`
- `testing-manual.md`

## 状态
- 更新日期: 2026-03-07
- 当前状态: active
- 下一任务: TASK-P2P-MLC-003
- 本轮完成:
  - 在 `agent_world_proto::viewer::AgentChatRequest` 增加 `intent_tick/intent_seq` 字段，并在 `AgentChatAck` 增加 `intent_tick/intent_seq/idempotent_replay`。
  - `runtime_live` 增加 `intent_seq` 幂等重放语义：同 `(player_id, agent_id, intent_seq)` 重试返回同 ACK，变更载荷触发冲突拒绝。
  - `viewer/auth` 的 agent_chat 签名载荷纳入 `intent_tick/intent_seq`，并校验 `intent_seq > 0`。
- 风险提示:
  - challenge 规则与实时体验存在冲突，需要联动客户端最终性文案。
  - 快照/日志可用性会直接影响重连追平成功率。
- 说明: 本文档只维护执行计划；过程记录写入 `doc/devlog/2026-03-07.md`。
