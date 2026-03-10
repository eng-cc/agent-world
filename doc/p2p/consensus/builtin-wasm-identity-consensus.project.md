# P2P Builtin Wasm 身份共识与跨平台构建（项目管理文档）

- 对应设计文档: `doc/p2p/consensus/builtin-wasm-identity-consensus.design.md`
- 对应需求文档: `doc/p2p/consensus/builtin-wasm-identity-consensus.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] BWI-1 设计文档落地（`doc/p2p/consensus/builtin-wasm-identity-consensus.prd.md`） (PRD-P2P-MIG-057)
- [x] BWI-2 项目管理文档落地（本文件） (PRD-P2P-MIG-057)
- [x] BWI-3 扩展 `scripts/sync-m1-builtin-wasm-artifacts.sh` (PRD-P2P-MIG-057)：生成/校验 identity manifest
- [x] BWI-4 扩展 `scripts/sync-m4-builtin-wasm-artifacts.sh` 与 `scripts/sync-m5-builtin-wasm-artifacts.sh` 复用 identity 生成/校验 (PRD-P2P-MIG-057)
- [x] BWI-5 runtime 接入 identity manifest（m1/m4/m5 builtin artifact loader） (PRD-P2P-MIG-057)
- [x] BWI-6 bootstrap 流程改造 (PRD-P2P-MIG-057)：`bootstrap_power/economy/gameplay` 使用 identity manifest 生成 `artifact_identity`
- [x] BWI-7 测试补齐 (PRD-P2P-MIG-057)：identity 解析/校验单测与 runtime 闭环测试
- [x] BWI-8 执行回归（required 相关命令）并更新文档/任务日志 (PRD-P2P-MIG-057)
- [x] BWI-9 扩展设计范围 (PRD-P2P-MIG-057)：纳入 CI/pre-commit/runtime fallback 全系统改造与旧方案归档
- [x] BWI-10 runtime materializer 生产化改造 (PRD-P2P-MIG-057)：补齐 m5 module-id 清单、支持非 canonical 平台回退编译缓存复用
- [x] BWI-11 门禁与流程改造 (PRD-P2P-MIG-057)：`scripts/ci-tests.sh required` 覆盖 `m1/m4/m5 --check`，文档与测试手册同步
- [x] BWI-13 回归收口 (PRD-P2P-MIG-057)：required tier + 定向 materializer 测试 + 清单校验 + 文档状态更新
- [x] BWI-14 维护收口 (PRD-P2P-MIG-057)：更新 m1/m5 hash 后补齐 m1/m4/m5 identity 重签，并同步 builtin signer 公钥

## 依赖
- `doc/p2p/consensus/builtin-wasm-identity-consensus.design.md`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`
- `scripts/sync-m5-builtin-wasm-artifacts.sh`
- `scripts/ci-tests.sh`
- `scripts/pre-commit.sh`
- `testing-manual.md`
- `crates/agent_world/src/runtime/builtin_wasm_materializer.rs`
- `crates/agent_world/src/runtime/m1_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/m4_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/m5_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/world/bootstrap_power.rs`
- `crates/agent_world/src/runtime/world/bootstrap_economy.rs`
- `crates/agent_world/src/runtime/world/bootstrap_gameplay.rs`

## 状态
- 当前阶段：已完成（BWI-14）
- 最近更新：BWI-14 完成，hash+identity+签名闭环回归通过（2026-02-27）
