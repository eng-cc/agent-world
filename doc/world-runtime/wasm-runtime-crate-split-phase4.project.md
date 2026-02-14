# Agent World Runtime：WASM 运行时激进迁移（项目管理文档，Phase 4）

## 任务拆解
- [x] R4-0 输出设计文档（`doc/world-runtime/wasm-runtime-crate-split-phase4.md`）
- [x] R4-0 输出项目管理文档（本文件）
- [ ] R4-1 迁移模块清单与变更计划类型到 `agent_world_wasm_abi` 并回归

## 依赖
- `crates/agent_world_wasm_abi/src/lib.rs`
- `crates/agent_world/src/runtime/modules.rs`
- `crates/agent_world/src/runtime/world/module_runtime.rs`

## 状态
- 当前阶段：R4-0 完成，R4-1 进行中。
- 最近更新：2026-02-14
