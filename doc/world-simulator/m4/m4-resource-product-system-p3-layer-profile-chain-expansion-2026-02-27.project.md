# M4 资源与产品系统 P3：分层档案化与链路扩展实现（项目管理文档）

## 任务拆解
- [x] T0：输出 P3 设计文档与项目管理文档。
- [ ] T1：落地 Profile 结构（ABI + WorldState）与默认目录注入。
- [ ] T2：接线 Profile 驱动规则（优先级、运损、阶段门槛、瓶颈标签、role_tag 优先级）。
- [ ] T3：扩展内置模块链路并同步 bootstrap/hash/identity 清单。
- [ ] T4：完成 required/full 回归，回写文档状态与 devlog 收口。

## 依赖
- `doc/world-simulator/m4/m4-resource-product-system-playability-2026-02-27.md`
- `doc/world-simulator/m4/m4-resource-product-system-p2-stage-guidance-market-governance-linkage-2026-02-27.md`
- `crates/agent_world_wasm_abi/src/economy.rs`
- `crates/agent_world/src/runtime/state.rs`
- `crates/agent_world/src/runtime/world/event_processing/action_to_event_core.rs`
- `crates/agent_world/src/runtime/world/event_processing/action_to_event_economy.rs`
- `crates/agent_world/src/runtime/world/bootstrap_economy.rs`
- `crates/agent_world/src/runtime/world/artifacts/m4_builtin_*`

## 状态
- 当前阶段：进行中（T0 已完成，执行 T1）。
- 阻塞项：无。
- 下一步：实现 Profile 数据结构并注入默认目录。
