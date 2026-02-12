# Agent World Runtime：Builtin 模块独立 Crate 化（BMS）

## 目标
- 将当前 `agent_world` 内部的 builtin 模块源码拆分到独立 crate，作为“Rust 源码形态的 wasm 模块仓”。
- 与已完成的 Rust->Wasm 构建套件对接，形成可重复的一键构建路径。
- 在不破坏现有 runtime 闭环的前提下，先完成“源码分仓 + 产物构建”，后续再推进运行时装载替换。

## 范围

### In Scope
- 新增独立 crate（workspace 成员）承载 builtin wasm 模块源码。
- 首批迁移 `m1.rule.move` 到独立 crate（保守增量）。
- 继续迁移规则模块 `m1.rule.visibility`、`m1.rule.transfer` 到独立 crate。
- 新增脚本封装，基于 `scripts/build-wasm-module.sh` 构建该 crate 产物。
- 补充最小测试与回归命令，确保 crate 可编译、脚本可执行、产物可生成。

### Out of Scope
- 本阶段不删除 `agent_world` 内现有 builtin sandbox 实现。
- 本阶段不切换 runtime 到“仅执行外部 wasm 产物”。
- 本阶段不覆盖全部 builtin（先迁移 `m1.rule.move` 样板）。

## 接口 / 数据
- 新增 crate（草案）：
  - `crates/agent_world_builtin_wasm`
  - 导出 wasm-1 ABI 入口：`alloc` + `reduce`（必要时兼容 `call`）
- 新增构建脚本（草案）：
  - `scripts/build-builtin-wasm-modules.sh`
  - 默认构建模块：`m1.rule.move`、`m1.rule.visibility`、`m1.rule.transfer`
  - 调用链路：`build-builtin-wasm-modules.sh -> build-wasm-module.sh -> wasm_build_suite`
- 产物目录（草案）：
  - `.tmp/builtin-wasm/<module-id>.wasm`
  - `.tmp/builtin-wasm/<module-id>.metadata.json`

## 里程碑
- M1：完成 BMS-1（独立 crate 初始化与 `m1.rule.move` wasm 模块样板）。
- M2：完成 BMS-2（构建脚本接入与验证）。
- M3：完成 BMS-3（回归验证、文档和 devlog 收口）。
- M4：完成 BMS-4~BMS-7（规则模块 `visibility/transfer` 迁移与回归收口）。

## 风险
- Rust 侧 wasm ABI 与 runtime 执行器签名（`(i32, i32) -> (i32, i32)`）存在兼容细节：通过定向测试覆盖。
- 独立 crate 与现有 builtin sandbox 会短期并行：需要在文档中明确“以 wasm 构建链路为先，运行时切换后续推进”。
- 后续批量迁移模块时会出现模块 ID 与版本治理一致性问题：本阶段先用单模块样板固化流程。
