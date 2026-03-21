# oasis7 Simulator：Rust 到 Wasm 编译套件（KWT）项目管理文档

- 对应设计文档: `doc/world-simulator/kernel/rust-wasm-build-suite.design.md`
- 对应需求文档: `doc/world-simulator/kernel/rust-wasm-build-suite.prd.md`

审计轮次: 5

## 任务拆解（含 PRD-ID 映射）
- [x] KWT-0 输出设计文档（`doc/world-simulator/kernel/rust-wasm-build-suite.prd.md`）与项目管理文档（本文件）。
- [x] KWT-1 新增 Rust->Wasm 构建套件（CLI + 脚本封装）。
- [x] KWT-2 补充构建套件测试与最小模板闭环。
- [x] KWT-3 回归验证、文档与 devlog 回写。

## 依赖
- `tools/`
- `scripts/`
- `crates/oasis7_wasm_executor/src/lib.rs`
- `doc/world-simulator/kernel/rust-wasm-build-suite.prd.md`

## 状态
- 当前阶段：已完成（KWT-0~KWT-3）
- 最近更新：完成 KWT-3（回归验证、文档与 devlog 收口，2026-02-12）。
- 下一步：按后续规则 wasm 模块演进需求，增量扩展模板与发布流水线。
