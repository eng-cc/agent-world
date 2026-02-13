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
- [x] BMS-17 迁移 `m1.mobility.basic` 到独立 wasm crate 并补充验证。
- [x] BMS-18 扩展构建脚本支持 `m1.mobility.basic` 并补充验证。
- [x] BMS-19 回归验证、文档与 devlog 收口。
- [x] BMS-20 扩展设计与任务拆解（`m1.memory.core` 迁移阶段）。
- [x] BMS-21 迁移 `m1.memory.core` 到独立 wasm crate 并补充验证。
- [x] BMS-22 扩展构建脚本支持 `m1.memory.core` 并补充验证。
- [x] BMS-23 回归验证、文档与 devlog 收口。
- [x] BMS-24 扩展设计与任务拆解（`m1.storage.cargo` 迁移阶段）。
- [x] BMS-25 迁移 `m1.storage.cargo` 到独立 wasm crate 并补充验证。
- [x] BMS-26 扩展构建脚本支持 `m1.storage.cargo` 并补充验证。
- [x] BMS-27 回归验证、文档与 devlog 收口。
- [x] BMS-28 扩展设计与任务拆解（`m1.power.radiation_harvest` / `m1.power.storage` 迁移阶段）。
- [x] BMS-29 迁移 `m1.power.radiation_harvest` / `m1.power.storage` 到独立 wasm crate 并补充验证。
- [x] BMS-30 扩展构建脚本支持 `m1.power.radiation_harvest` / `m1.power.storage` 并补充验证。
- [x] BMS-31 回归验证、文档与 devlog 收口。
- [x] BMS-32 扩展设计与任务拆解（runtime cutover：WASM 优先 + builtin fallback + 渐进下线 builtin 注册）。
- [x] BMS-33 实现 runtime 执行路径切换（WASM 优先 + builtin fallback）并补充验证。
- [x] BMS-34 逐步下线一批 builtin 注册点（先 tests/demo）并补充验证。
- [x] BMS-35 回归验证、文档与 devlog 收口（cutover 阶段一期）。
- [x] BMS-36 扩展设计与任务拆解（cutover 阶段二：逐域删除 runtime builtin fallback/实现）。
- [x] BMS-37 下线 `rule/body` 相关 builtin 测试注册与默认执行路径（wasm 工件优先）。
- [x] BMS-38 下线 `sensor/mobility/memory/storage/power` 相关 builtin 测试注册与默认执行路径（wasm 工件优先）。
- [x] BMS-39 清理 runtime 中不再使用的 builtin 模块实现导出与冗余回退路径，并完成回归收口。
- [x] BMS-40 扩展设计与任务拆解（阶段三启动：逐步物理删除 native builtin 老代码）。
- [x] BMS-41 删除 `runtime/builtin_modules/*` native 实现文件，保留运行时常量与最小 sandbox 兼容层。
- [x] BMS-42 下线 `BuiltinModuleSandbox` 的 builtin 注册兜底能力并清理残余引用。
- [x] BMS-43 回归验证、文档与 devlog 收口（阶段三首轮）。
- [x] BMS-44 扩展设计与任务拆解（阶段三第二轮：删除 `BuiltinModuleSandbox` 兼容层及导出）。
- [x] BMS-45 删除 `BuiltinModuleSandbox` 类型与 `runtime` 对外导出，保留模块常量导出。
- [x] BMS-46 回归验证、文档与 devlog 收口（阶段三第二轮）。
- [x] BMS-47 扩展设计与任务拆解（阶段三第三轮：删除 runtime builtin 常量兼容层）。
- [ ] BMS-48 删除 `runtime/builtin_modules.rs` 常量层，统一引用 `agent_world_builtin_wasm` 常量导出。
- [ ] BMS-49 回归验证、文档与 devlog 收口（阶段三第三轮）。

## 依赖
- `tools/wasm_build_suite`
- `scripts/build-wasm-module.sh`
- `crates/agent_world`（现有 builtin 行为作为对照）

## 状态
- 当前阶段：cutover 阶段三第三轮进行中（BMS-47 已完成，进入常量层删除实现阶段）。
- 最近更新：完成 BMS-47（阶段三第三轮任务拆解，2026-02-13）。
- 下一步：执行 BMS-48，删除 `runtime/builtin_modules.rs` 并统一常量来源。
