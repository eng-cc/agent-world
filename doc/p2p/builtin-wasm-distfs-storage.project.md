# World Runtime：Builtin Wasm DistFS 存储与提交前校验（项目管理文档）

## 任务拆解
- [x] BWD-1 输出设计文档（`doc/p2p/builtin-wasm-distfs-storage.md`）与项目管理文档（本文件）。
- [x] BWD-2 改造 wasm 同步脚本：以 hash 清单为 git 基线，产物落盘到 DistFS 本地存储。
- [x] BWD-3 改造 pre-commit：提交前执行 builtin wasm 校验。
- [x] BWD-4 改造 runtime builtin wasm 加载路径：DistFS 读取 + hash 校验。
- [x] BWD-5 移除 wasm 二进制 git 追踪，更新 ignore 规则。
- [x] BWD-6 执行 `test_tier_required` 回归并完成文档/devlog 收口。

## 依赖
- `scripts/sync-m1-builtin-wasm-artifacts.sh`
- `scripts/sync-m4-builtin-wasm-artifacts.sh`
- `scripts/pre-commit.sh`
- `scripts/ci-tests.sh`
- `crates/agent_world/src/runtime/m1_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/m4_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/world/bootstrap_power.rs`
- `crates/agent_world/src/runtime/world/bootstrap_economy.rs`
- `crates/agent_world/src/runtime/world/artifacts/*.sha256`

## 状态
- 当前阶段：已完成。
- 最近更新：完成 BWD-2~BWD-6（脚本/runtime/git 追踪改造与 required tier 回归，2026-02-15）。
