# Agent World Runtime：Builtin 模块独立 Crate 化（BMS）

## 目标
- 将当前 `agent_world` 内部的 builtin 模块源码拆分到独立 crate，作为“Rust 源码形态的 wasm 模块仓”。
- 与已完成的 Rust->Wasm 构建套件对接，形成可重复的一键构建路径。
- 在不破坏现有 runtime 闭环的前提下，分阶段完成：
  - 阶段 1：`源码分仓 + 产物构建`（BMS-0~BMS-31，已完成）。
  - 阶段 2：runtime 装载切换（外部 wasm 产物优先，builtin 仅兜底，逐步删除 builtin 注册）。
  - 阶段 3：逐域删除 runtime builtin fallback 与实现（以 wasm-only 运行为目标）。

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
- 新增 runtime cutover 路径：
  - 在模块执行链路中接入“WASM 优先 + builtin fallback”。
  - 在测试/示例安装入口逐步下线 builtin 注册，改为安装 wasm 模块工件。
  - 保持灰度期兼容，最终移除 builtin-only 执行路径。

### Out of Scope
- 不在单一任务内一次性删除所有 builtin（仅按批次渐进删除）。
- 不引入额外模块语义变更（只做执行载体切换，不改变规则语义）。

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
- runtime 装载接口（阶段三现状）：
  - `BuiltinModuleSandbox` 兼容层已在 BMS-45 删除。
  - 默认执行链路为 wasm 工件路径（`WasmExecutor`）。
  - `runtime` 侧 builtin 常量兼容层已在 BMS-48 删除，常量统一来自 `agent_world_builtin_wasm`。
- 阶段三下线路线（2026-02-13）：
  - BMS-40：补充阶段三任务拆解，明确“先删实现、后删接口、最后收口”节奏。
  - BMS-41：物理删除 `runtime/builtin_modules/` 下 `rule/body/default/power` native 实现文件，仅保留模块 ID/版本/参数常量。
  - BMS-42：下线 `BuiltinModuleSandbox` 的 builtin 注册兜底能力，收敛到 wasm-only 执行链路。
  - BMS-43：执行阶段三首轮回归收口，更新文档与 devlog。
- 阶段三第二轮下线路线（2026-02-13）：
  - BMS-44：扩展任务拆解，明确移除 `BuiltinModuleSandbox` 兼容层与导出的顺序。
  - BMS-45：移除 `BuiltinModuleSandbox` 类型与 `runtime` 对外导出，仅保留模块常量导出。
  - BMS-46：执行第二轮回归收口，更新文档与 devlog。
- 阶段三第三轮下线路线（2026-02-13）：
  - BMS-47：扩展任务拆解，明确“删除 runtime builtin 常量兼容层并统一常量来源”的执行顺序。
  - BMS-48：移除 `runtime/builtin_modules.rs`，将常量来源统一到 `agent_world_builtin_wasm` crate。
  - BMS-49：执行第三轮回归收口，更新文档与 devlog。
- 阶段四收尾路线（2026-02-13）：
  - BMS-50：扩展任务拆解，明确“产物接入收敛 + 文档去陈旧 + 渐进工件策略”的执行顺序。
  - BMS-51：清理过时文档描述（`BuiltinModuleSandbox`/`runtime/builtin_modules.rs` 等），统一到当前 wasm-only 事实。
  - BMS-52：补齐 runtime 内嵌 wasm 工件同步机制（构建 -> 回填 -> 哈希校验），避免源码与工件漂移。
  - BMS-53：收敛 bootstrap/tests 的工件引用入口，减少 `include_bytes!(m1_builtin_modules.wasm)` 分散硬编码。
  - BMS-54：评估并决策“单聚合 wasm 工件 vs 多模块独立 wasm 工件”，输出迁移方案与分批落地顺序。
  - BMS-55：执行阶段四回归收口，更新文档与 devlog。

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
- M11：完成 BMS-32（runtime cutover 设计与任务拆解扩展）。
- M12：完成 BMS-33（接入 WASM 优先 + builtin fallback 执行路径）。
- M13：完成 BMS-34（逐步缩减一批 builtin 注册点，保持回归通过）。
- M14：完成 BMS-35（cutover 阶段回归收口与文档闭环）。
- M15：完成 BMS-36（cutover 阶段二设计扩展与任务拆解）。
- M16：完成 BMS-37~BMS-39（按模块域逐步删除 builtin fallback 与实现）。
- M17：完成 BMS-40~BMS-43（阶段三首轮：物理删除 native builtin 老代码并收口接口）。
- M18：完成 BMS-44~BMS-46（阶段三第二轮：删除兼容 sandbox 层并收口导出）。
- M19：完成 BMS-47~BMS-49（阶段三第三轮：删除 runtime builtin 常量兼容层并统一常量来源）。
- M20：完成 BMS-50~BMS-55（阶段四：产物接入收敛、文档去陈旧与后续工件迁移决策）。

## 风险
- Rust 侧 wasm ABI 与 runtime 执行器签名（`(i32, i32) -> (i32, i32)`）存在兼容细节：通过定向测试覆盖。
- 独立 crate 与现有 builtin sandbox 会短期并行：需要在文档中明确“以 wasm 构建链路为先，运行时切换后续推进”。
- 后续批量迁移模块时会出现模块 ID 与版本治理一致性问题：本阶段先用单模块样板固化流程。
- 默认模块封装层（如 `m1.mobility.basic`）复用底层规则模块时，存在行为漂移风险：通过并行单测对齐 native 与 wasm 输出。
- 状态型模块（如 `m1.memory.core`）依赖事件解析与窗口裁剪，存在状态编码兼容风险：通过状态 round-trip 与事件序列单测覆盖。
- 账本型模块（如 `m1.storage.cargo`）依赖多类领域事件聚合与饱和计数，存在事件映射遗漏风险：通过成功/拒绝路径和状态增量单测覆盖。
- 资源型模块（如 `m1.power.radiation_harvest` / `m1.power.storage`）依赖“采集-扣费-位置更新”复合路径，存在动作与事件双入口行为偏差风险：通过动作驱动与事件驱动并行测试覆盖。
- runtime 双路径（wasm + builtin fallback）短期并行时，存在“同模块不同执行器”导致行为漂移风险：通过同输入对比测试与回放一致性测试覆盖。
- 逐步删除 builtin 注册点过程中，存在测试夹具未同步导致模块缺失风险：先在测试入口灰度切换，保留 fallback 并补充缺失报错断言。
- 删除 runtime 内 builtin 实现时，存在“无 wasmtime 构建路径”兼容风险：通过 feature 门控分阶段下线，并在每批任务执行双路径回归（with/without wasmtime）。
- 物理删除 `runtime/builtin_modules/*` 旧实现后，存在隐藏引用或常量来源漂移风险：通过 `rg` 全量扫描 + 定向编译测试 + bootstrap/power 回归覆盖。
- 下线 builtin 注册 API 时，存在测试夹具未同步导致不可执行风险：阶段三保持“每删一层就补对应测试迁移”的原子任务提交。
- 删除 `BuiltinModuleSandbox` 导出后，存在下游调用方编译失败风险：先 `rg` 扫描确认无代码引用，再执行 wasm 路径回归。
- 删除 `runtime/builtin_modules.rs` 后，存在常量可见性与依赖方向变化风险：先在 wasm crate 补齐 `pub const` 导出，再执行 bootstrap/rules/power 路径回归。
- 当前 runtime/测试仍依赖 `m1_builtin_modules.wasm` 单工件硬编码，存在“源码更新但工件未同步”风险：通过构建后哈希校验与预提交检查降低漂移概率。
- 若切到多模块独立 wasm 工件，存在“加载入口与模块治理流程复杂度上升”风险：先做决策文档与 PoC，再按域分批迁移。
