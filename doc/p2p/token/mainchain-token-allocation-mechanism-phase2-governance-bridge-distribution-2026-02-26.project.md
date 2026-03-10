# Agent World 主链 Token 分配机制二期（项目管理文档）

- 对应设计文档: `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.design.md`
- 对应需求文档: `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-P2P-MIG-111)：设计文档与项目管理文档建档。
- [x] T1 (PRD-P2P-MIG-111)：实现 Node -> 主链 Token 地址绑定模型并接入 NodePoints 桥接。
- [x] T2 (PRD-P2P-MIG-111)：实现主链策略更新与治理提案生命周期绑定校验。
- [x] T3 (PRD-P2P-MIG-111)：实现 staking/ecosystem/security treasury 分发动作闭环与审计记录。
- [x] T4 (PRD-P2P-MIG-111)：补齐测试、回归脚本验证与文档回写收口。

## 依赖
- doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.md
- `doc/p2p/token/mainchain-token-allocation-mechanism.prd.md`
- `doc/p2p/token/mainchain-token-allocation-mechanism.project.md`
- `crates/agent_world/src/runtime/main_token.rs`
- `crates/agent_world/src/runtime/events.rs`
- `crates/agent_world/src/runtime/world/event_processing.rs`
- `crates/agent_world/src/runtime/state.rs`
- `crates/agent_world/src/runtime/state/apply_domain_event_main_token.rs`
- `crates/agent_world/src/runtime/world/resources.rs`
- `crates/agent_world/src/runtime/tests/main_token.rs`
- `crates/agent_world/src/runtime/tests/reward_asset_settlement_action.rs`

## 状态
- 当前阶段：T0~T4 全部完成。
- 下一步：无（进入后续迭代需求）。
- 最近更新：2026-02-27。
