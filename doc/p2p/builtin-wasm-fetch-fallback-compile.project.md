# World Runtime：Builtin Wasm 先拉取后编译回退（项目管理文档）

## 任务拆解
- [x] BFC-1 设计文档落地（`doc/p2p/builtin-wasm-fetch-fallback-compile.md`）
- [x] BFC-1 项目管理文档落地（本文件）
- [x] BFC-2 实现 runtime builtin wasm 装载链路：本地命中 -> 网络拉取 -> 本地编译回退
- [x] BFC-3 增加 `test_tier_full` 闭环测试（拉取失败回退编译）
- [x] BFC-4 执行回归并更新 devlog
- [x] BFC-5 修复 viewer wasm32 编译兼容：`builtin_wasm_materializer` 在 `wasm32` 下禁用 `reqwest::blocking` HTTP 拉取分支

## 依赖
- `crates/agent_world/src/runtime/m1_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/m4_builtin_wasm_artifact.rs`
- `crates/agent_world/src/runtime/mod.rs`
- `crates/agent_world/src/runtime/builtin_wasm_materializer.rs`
- `crates/agent_world/src/runtime/tests/*`
- `scripts/build-wasm-module.sh`
- `tools/wasm_build_suite/*`

## 状态
- 当前阶段：已完成（含 wasm32 编译兼容修复）
