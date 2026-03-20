# World Runtime：Builtin Wasm 先拉取后编译回退（项目管理文档）

- 对应设计文档: `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.design.md`
- 对应需求文档: `doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] BFC-1 设计文档落地（`doc/p2p/node/node-builtin-wasm-fetch-fallback-compile.prd.md`） (PRD-P2P-MIG-087)
- [x] BFC-1 项目管理文档落地（本文件） (PRD-P2P-MIG-087)
- [x] BFC-2 实现 runtime builtin wasm 装载链路 (PRD-P2P-MIG-087)：本地命中 -> 网络拉取 -> 本地编译回退
- [x] BFC-3 增加 `test_tier_full` 闭环测试（拉取失败回退编译） (PRD-P2P-MIG-087)
- [x] BFC-4 执行回归并更新 devlog (PRD-P2P-MIG-087)
- [x] BFC-5 修复 viewer wasm32 编译兼容 (PRD-P2P-MIG-087)：`builtin_wasm_materializer` 在 `wasm32` 下禁用 `reqwest::blocking` HTTP 拉取分支

## 依赖
- `crates/oasis7/src/runtime/m1_builtin_wasm_artifact.rs`
- `crates/oasis7/src/runtime/m4_builtin_wasm_artifact.rs`
- `crates/oasis7/src/runtime/mod.rs`
- `crates/oasis7/src/runtime/builtin_wasm_materializer.rs`
- `crates/oasis7/src/runtime/tests/*`
- `scripts/build-wasm-module.sh`
- `tools/wasm_build_suite/*`

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- 当前阶段：已完成（含 wasm32 编译兼容修复）
