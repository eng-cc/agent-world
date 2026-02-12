# Agent World Runtime：Builtin 模块独立 Crate 化（BMS）项目管理文档

## 任务拆解
- [x] BMS-0 输出设计文档（`doc/world-runtime/builtin-wasm-crate-split.md`）与项目管理文档（本文件）。
- [x] BMS-1 新增独立 crate 并迁移首个 builtin wasm 模块（`m1.rule.move`）。
- [ ] BMS-2 接入构建脚本（调用 Rust->Wasm 构建套件）并补充验证。
- [ ] BMS-3 回归验证、文档与 devlog 收口。

## 依赖
- `tools/wasm_build_suite`
- `scripts/build-wasm-module.sh`
- `crates/agent_world`（现有 builtin 行为作为对照）

## 状态
- 当前阶段：BMS-2（进行中）
- 最近更新：完成 BMS-1（独立 crate + `m1.rule.move` wasm 样板，2026-02-12）。
- 下一步：接入构建脚本并补充验证。
