# Agent World 主链 Token 分配机制二期（项目管理文档）

## 任务拆解
- [x] T0：设计文档与项目管理文档建档。
- [x] T1：实现 Node -> 主链 Token 地址绑定模型并接入 NodePoints 桥接。
- [x] T2：实现主链策略更新与治理提案生命周期绑定校验。
- [x] T3：实现 staking/ecosystem/security treasury 分发动作闭环与审计记录。
- [ ] T4：补齐测试、回归脚本验证与文档回写收口。

## 依赖
- `doc/p2p/mainchain-token-allocation-mechanism.md`
- `doc/p2p/mainchain-token-allocation-mechanism.project.md`
- `crates/agent_world/src/runtime/main_token.rs`
- `crates/agent_world/src/runtime/events.rs`
- `crates/agent_world/src/runtime/world/event_processing.rs`
- `crates/agent_world/src/runtime/state.rs`
- `crates/agent_world/src/runtime/state/apply_domain_event_main_token.rs`
- `crates/agent_world/src/runtime/world/resources.rs`
- `crates/agent_world/src/runtime/tests/main_token.rs`
- `crates/agent_world/src/runtime/tests/reward_asset_settlement_action.rs`

## 状态
- 当前阶段：T0~T3 已完成，进入 T4。
- 下一步：补齐测试、回归脚本验证与文档回写收口。
- 最近更新：2026-02-26。
