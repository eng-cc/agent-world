# Agent World Simulator：Rust 到 Wasm 编译套件（KWT）项目管理文档

## 任务拆解
- [x] KWT-0 输出设计文档（`doc/world-simulator/rust-wasm-build-suite.md`）与项目管理文档（本文件）。
- [ ] KWT-1 新增 Rust->Wasm 构建套件（CLI + 脚本封装）。
- [ ] KWT-2 补充构建套件测试与最小模板闭环。
- [ ] KWT-3 回归验证、文档与 devlog 回写。

## 依赖
- `tools/`
- `scripts/`
- `crates/agent_world/src/runtime/sandbox.rs`
- `doc/world-simulator/rust-wasm-build-suite.md`

## 状态
- 当前阶段：KWT-1（进行中）
- 最近更新：完成 KWT-0（设计文档与项目管理文档，2026-02-12）。
- 下一步：实现 Rust->Wasm 构建套件主功能（CLI + 脚本封装）。
