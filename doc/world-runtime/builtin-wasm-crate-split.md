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
- 继续迁移 `m1.body.core` 到独立 crate。
- 继续迁移 `m1.sensor.basic` 到独立 crate。
- 继续迁移 `m1.mobility.basic` 到独立 crate。
- 继续迁移 `m1.memory.core` 到独立 crate。
- 继续迁移 `m1.storage.cargo` 到独立 crate。
- 继续迁移 `m1.power.radiation_harvest` 到独立 crate。
- 继续迁移 `m1.power.storage` 到独立 crate。
- 新增脚本封装，基于 `scripts/build-wasm-module.sh` 构建该 crate 产物。
- 补充最小测试与回归命令，确保 crate 可编译、脚本可执行、产物可生成。

### Out of Scope
- 本阶段不删除 `agent_world` 内现有 builtin sandbox 实现。
- 本阶段不切换 runtime 到“仅执行外部 wasm 产物”。
- 本阶段不覆盖全部 builtin（按优先级增量迁移）。

## 接口 / 数据
- 新增 crate（草案）：
  - `crates/agent_world_builtin_wasm`
  - 导出 wasm-1 ABI 入口：`alloc` + `reduce`（必要时兼容 `call`）
- 新增构建脚本（草案）：
  - `scripts/build-builtin-wasm-modules.sh`
  - 默认构建模块：`m1.rule.move`、`m1.rule.visibility`、`m1.rule.transfer`、`m1.body.core`、`m1.sensor.basic`、`m1.mobility.basic`、`m1.memory.core`、`m1.storage.cargo`、`m1.power.radiation_harvest`、`m1.power.storage`
  - 调用链路：`build-builtin-wasm-modules.sh -> build-wasm-module.sh -> wasm_build_suite`
- 产物目录（草案）：
  - `.tmp/builtin-wasm/<module-id>.wasm`
  - `.tmp/builtin-wasm/<module-id>.metadata.json`

## 里程碑
- M1：完成 BMS-1（独立 crate 初始化与 `m1.rule.move` wasm 模块样板）。
- M2：完成 BMS-2（构建脚本接入与验证）。
- M3：完成 BMS-3（回归验证、文档和 devlog 收口）。
- M4：完成 BMS-4~BMS-7（规则模块 `visibility/transfer` 迁移与回归收口）。
- M5：完成 BMS-8~BMS-11（`m1.body.core` 迁移与回归收口）。
- M6：完成 BMS-12~BMS-15（`m1.sensor.basic` 迁移与回归收口）。
- M7：完成 BMS-16~BMS-19（`m1.mobility.basic` 迁移与回归收口）。
- M8：完成 BMS-20~BMS-23（`m1.memory.core` 迁移与回归收口）。
- M9：完成 BMS-24~BMS-27（`m1.storage.cargo` 迁移与回归收口）。
- M10：完成 BMS-28~BMS-31（`m1.power.radiation_harvest` / `m1.power.storage` 迁移与回归收口）。

## 风险
- Rust 侧 wasm ABI 与 runtime 执行器签名（`(i32, i32) -> (i32, i32)`）存在兼容细节：通过定向测试覆盖。
- 独立 crate 与现有 builtin sandbox 会短期并行：需要在文档中明确“以 wasm 构建链路为先，运行时切换后续推进”。
- 后续批量迁移模块时会出现模块 ID 与版本治理一致性问题：本阶段先用单模块样板固化流程。
- 默认模块封装层（如 `m1.mobility.basic`）复用底层规则模块时，存在行为漂移风险：通过并行单测对齐 native 与 wasm 输出。
- 状态型模块（如 `m1.memory.core`）依赖事件解析与窗口裁剪，存在状态编码兼容风险：通过状态 round-trip 与事件序列单测覆盖。
- 账本型模块（如 `m1.storage.cargo`）依赖多类领域事件聚合与饱和计数，存在事件映射遗漏风险：通过成功/拒绝路径和状态增量单测覆盖。
- 资源型模块（如 `m1.power.radiation_harvest` / `m1.power.storage`）依赖“采集-扣费-位置更新”复合路径，存在动作与事件双入口行为偏差风险：通过动作驱动与事件驱动并行测试覆盖。
