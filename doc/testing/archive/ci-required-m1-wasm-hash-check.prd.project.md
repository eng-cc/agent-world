# Required Tier 接入 M1 Builtin Wasm Hash 校验（项目管理文档）

> 归档说明（2026-02-20）：该任务已由 `doc/p2p/consensus/builtin-wasm-identity-consensus.prd.md` / `.project.md` 覆盖并替代。

## 任务拆解（含 PRD-ID 映射）
- [x] ARCH-M1-1 (PRD-TESTING-ARCHIVE-M1-001): 完成专题设计与项目管理建档。
- [x] ARCH-M1-2 (PRD-TESTING-ARCHIVE-M1-001/002): 在 `scripts/ci-tests.sh` required/full 前置接入 `sync-m1 --check`。
- [x] ARCH-M1-3 (PRD-TESTING-ARCHIVE-M1-002/003): 升级 manifest 为多平台多 hash 格式并同步 `sync-m1` 校验逻辑。
- [x] ARCH-M1-4 (PRD-TESTING-ARCHIVE-M1-002/003): 改造 DistFS hydration 与 runtime materializer 候选 hash 匹配并完成回归。
- [x] ARCH-M1-5 (PRD-TESTING-004): 归档专题文档迁移为 strict schema 与 `.prd.md/.prd.project.md` 命名。

## 依赖
- `scripts/ci-tests.sh`
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256`
- `crates/agent_world_distfs/src/bin/hydrate_builtin_wasm.rs`
- `crates/agent_world/src/runtime/builtin_wasm_materializer.rs`
- `doc/testing/prd.md`
- `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已归档（由 builtin wasm identity 新方案替代）
- 阻塞项：无
- 下一步：无
