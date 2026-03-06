# Agent World: Builtin Wasm 构建确定性护栏（项目管理）

审计轮次: 3

## 任务拆解（含 PRD-ID 映射）
- [x] WASMDET-1 (PRD-TESTING-GOV-WASMDET-001/003): 完成专题设计文档与项目管理文档基线。
- [x] WASMDET-2 (PRD-TESTING-GOV-WASMDET-001/002): 在 `scripts/build-wasm-module.sh` 落地 canonical 输入约束、污染环境变量拦截、复现环境固定。
- [x] WASMDET-3 (PRD-TESTING-GOV-WASMDET-002/003): 在 `tools/wasm_build_suite/src/lib.rs` 增加 `--locked` 与 workspace 编译期拦截，并完成测试回归。
- [x] WASMDET-4 (PRD-TESTING-004): 专题文档按 strict schema 人工重写，并切换命名到 `.prd.md/.prd.project.md`。

## 依赖
- doc/testing/governance/wasm-build-determinism-guard.prd.md
- wasm 构建入口：`scripts/build-wasm-module.sh`
- wasm 构建工具：`tools/wasm_build_suite/src/lib.rs`
- required 门禁入口：`scripts/ci-tests.sh`
- hash 工件检查：
  - `scripts/sync-m1-builtin-wasm-artifacts.sh`
  - `scripts/sync-m4-builtin-wasm-artifacts.sh`
- 模块追踪文档：
  - `doc/testing/prd.md`
  - `doc/testing/prd.project.md`

## 状态
- 更新日期：2026-03-03
- 当前阶段：已完成
- 阻塞项：无
- 下一步：无（当前专题已收口）
