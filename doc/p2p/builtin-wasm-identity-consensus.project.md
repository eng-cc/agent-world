# P2P Builtin Wasm 身份共识与跨平台构建（项目管理文档）

## 任务拆解
- [x] BWI-1 设计文档落地（`doc/p2p/builtin-wasm-identity-consensus.md`）
- [x] BWI-2 项目管理文档落地（本文件）
- [x] BWI-3 扩展 `scripts/sync-m1-builtin-wasm-artifacts.sh`：生成/校验 identity manifest
- [x] BWI-4 扩展 `scripts/sync-m4-builtin-wasm-artifacts.sh` 与 `scripts/sync-m5-builtin-wasm-artifacts.sh` 复用 identity 生成/校验
- [x] BWI-5 runtime 接入 identity manifest（m1/m4/m5 builtin artifact loader）
- [x] BWI-6 bootstrap 流程改造：`bootstrap_power/economy/gameplay` 使用 identity manifest 生成 `artifact_identity`
- [x] BWI-7 测试补齐：identity 解析/校验单测与 runtime 闭环测试
- [x] BWI-8 执行回归（required 相关命令）并更新文档/任务日志

## 依赖
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`
- `scripts/sync-m5-builtin-wasm-artifacts.sh`
- `crates/agent_world/src/runtime/m1_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/m4_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/m5_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/world/bootstrap_power.rs`
- `crates/agent_world/src/runtime/world/bootstrap_economy.rs`
- `crates/agent_world/src/runtime/world/bootstrap_gameplay.rs`

## 状态
- 当前阶段：已完成
- 最近更新：BWI-8 完成（2026-02-20）
