# Agent World Runtime：Builtin 模块独立 Crate 化（BMS）项目管理文档

## 任务拆解
- [x] BMS-0 输出设计文档（`doc/world-runtime/builtin-wasm-crate-split.md`）与项目管理文档（本文件）。
- [x] BMS-1 新增独立 crate 并迁移首个 builtin wasm 模块（`m1.rule.move`）。
- [x] BMS-2 接入构建脚本（调用 Rust->Wasm 构建套件）并补充验证。
- [x] BMS-3 回归验证、文档与 devlog 收口。
- [x] BMS-4 扩展设计与任务拆解（`m1.rule.visibility` / `m1.rule.transfer` 迁移阶段）。
- [x] BMS-5 迁移 `m1.rule.visibility` 到独立 wasm crate 并补充验证。
- [x] BMS-6 迁移 `m1.rule.transfer` 到独立 wasm crate，扩展构建脚本并补充验证。
- [x] BMS-7 回归验证、文档与 devlog 收口。
- [x] BMS-8 扩展设计与任务拆解（`m1.body.core` 迁移阶段）。
- [x] BMS-9 迁移 `m1.body.core` 到独立 wasm crate 并补充验证。
- [x] BMS-10 扩展构建脚本支持 `m1.body.core` 并补充验证。
- [x] BMS-11 回归验证、文档与 devlog 收口。
- [x] BMS-12 扩展设计与任务拆解（`m1.sensor.basic` 迁移阶段）。
- [x] BMS-13 迁移 `m1.sensor.basic` 到独立 wasm crate 并补充验证。
- [x] BMS-14 扩展构建脚本支持 `m1.sensor.basic` 并补充验证。
- [x] BMS-15 回归验证、文档与 devlog 收口。
- [x] BMS-16 扩展设计与任务拆解（`m1.mobility.basic` 迁移阶段）。
- [ ] BMS-17 迁移 `m1.mobility.basic` 到独立 wasm crate 并补充验证。
- [ ] BMS-18 扩展构建脚本支持 `m1.mobility.basic` 并补充验证。
- [ ] BMS-19 回归验证、文档与 devlog 收口。

## 依赖
- `tools/wasm_build_suite`
- `scripts/build-wasm-module.sh`
- `crates/agent_world`（现有 builtin 行为作为对照）

## 状态
- 当前阶段：BMS-17（进行中）
- 最近更新：完成 BMS-16（`m1.mobility.basic` 迁移阶段文档拆解，2026-02-12）。
- 下一步：完成 BMS-17（迁移 `m1.mobility.basic` 到独立 wasm crate 并补充验证）。
