# Builtin Wasm 平台 Canonical Hash 清单（项目管理文档）

> 归档说明（2026-02-20）：该任务已由 `doc/p2p/builtin-wasm-identity-consensus.md` / `.project.md` 覆盖并替代。

## 任务拆解（含 PRD-ID 映射）
- [x] ARCH-CANON-1 (PRD-TESTING-ARCHIVE-CANON-001): 完成专题设计与项目管理建档。
- [x] ARCH-CANON-2 (PRD-TESTING-ARCHIVE-CANON-001/003): 改造 `sync-m1` 为平台 canonical 校验与更新策略。
- [x] ARCH-CANON-3 (PRD-TESTING-ARCHIVE-CANON-002): runtime/hydrator/测试支持 `platform=hash` 并兼容 legacy token。
- [x] ARCH-CANON-4 (PRD-TESTING-ARCHIVE-CANON-002/003): 迁移 m1 manifest 数据并完成 required 回归。
- [x] ARCH-CANON-5 (PRD-TESTING-004): 归档专题文档迁移为 strict schema 与 `.prd.md/.prd.project.md` 命名。

## 依赖
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/ci-tests.sh`
- `crates/agent_world/src/runtime/m1_builtin_wasm_artifact.rs`
- `crates/agent_world_distfs/src/bin/hydrate_builtin_wasm.rs`
- `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已归档（由 builtin wasm identity 新方案替代）
- 阻塞项：无
- 下一步：无
