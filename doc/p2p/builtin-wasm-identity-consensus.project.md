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
- [x] BWI-9 扩展设计范围：纳入 CI/pre-commit/runtime fallback 全系统改造与旧方案归档
- [x] BWI-10 runtime materializer 生产化改造：补齐 m5 module-id 清单、支持非 canonical 平台回退编译缓存复用
- [x] BWI-11 门禁与流程改造：`scripts/ci-tests.sh required` 覆盖 `m1/m4/m5 --check`，文档与测试手册同步
- [x] BWI-12 过时方案归档：将 hash-only 旧设计文档迁移到 `archive/` 并修复活跃引用
- [ ] BWI-13 回归收口：required tier + 定向 materializer 测试 + 清单校验 + 文档状态更新

## 依赖
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
- 当前阶段：进行中（BWI-13）
- 最近更新：BWI-12 完成，进入回归收口阶段（2026-02-20）
