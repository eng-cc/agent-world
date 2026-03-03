# World Runtime: Wasm 构建一致性与模块身份升级（项目管理文档）

## 任务拆解（含 PRD-ID 映射）
- [x] WIR-1 设计文档落地（`doc/p2p/wasm-artifact-identity-reproducibility.md`） (PRD-P2P-MIG-045)
- [x] WIR-1 项目管理文档落地（本文件） (PRD-P2P-MIG-045)
- [x] WIR-2 固定 toolchain（`rust-toolchain.toml`）并对齐 CI workflow (PRD-P2P-MIG-045)
- [x] WIR-3 新增模块身份结构并接入 `ModuleManifest` (PRD-P2P-MIG-045)
- [x] WIR-3 runtime 增加身份完整性校验并补齐测试 (PRD-P2P-MIG-045)
- [x] WIR-4 更新任务日志并完成回归 (PRD-P2P-MIG-045)

## 依赖
- `.github/workflows/rust.yml`
- `crates/agent_world_wasm_abi/src/lib.rs`
- `crates/agent_world/src/runtime/world/module_runtime.rs`
- `crates/agent_world/src/runtime/world/bootstrap_power.rs`
- `crates/agent_world/src/runtime/world/bootstrap_economy.rs`
- `crates/agent_world_wasm_store/src/lib.rs`

## 状态
- 当前阶段：已完成
