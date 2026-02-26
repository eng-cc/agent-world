# Agent World Runtime：零信任多节点治理与签名加固（项目管理）

## 任务拆解
- [x] T0 建立设计文档与任务拆解
- [ ] T1 P0 工件真实性链（artifact_identity 必填 + 禁止 unsigned + 注册/加载验签）
- [ ] T2 P0 治理共识绑定与原子 apply（最终性证书 + 多签门限）
- [ ] T3 P1 收据签名升级（节点签名/阈值签名 + 共识高度锚定）
- [ ] T4 P1 执行错误可观测性（OutOfFuel/Interrupt 区分）
- [ ] T5 回归测试、文档回写、devlog 收口

## 依赖
- `doc/world-runtime/zero-trust-governance-receipt-hardening-2026-02-26.md`
- `crates/agent_world/src/runtime/world/base_layer.rs`
- `crates/agent_world/src/runtime/world/persistence.rs`
- `crates/agent_world/src/runtime/world/governance.rs`
- `crates/agent_world/src/runtime/signer.rs`
- `crates/agent_world/src/runtime/effect.rs`
- `crates/agent_world_wasm_executor/src/lib.rs`

## 状态
- 当前阶段：T1 进行中（P0 工件真实性链）
