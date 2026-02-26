# M4 内置 WASM 模块可维护性收口（2026-02-26）

## 目标
- 在不改变现有 M4 经济模块行为与协议的前提下，完成两项工程收口：
  - 模块模板化抽象：减少 `Recipe/Product/Factory` 三类模块重复代码。
  - Bootstrap 清单驱动化：去掉 `install_m4_economy_bootstrap_modules` 的手工逐项注册。
- 增加一致性护栏，避免后续新增模块时出现“代码清单/运行时清单/工件清单”漂移。

## 范围
### In Scope
- `crates/agent_world_builtin_wasm_modules/m4_*` 的模块入口重构（模板化）。
- `crates/agent_world/src/runtime/world/bootstrap_economy.rs` 改为描述符清单驱动。
- `runtime` 侧一致性校验与测试补充。
- 文档、项目管理文档、devlog 更新。

### Out of Scope
- M1/M5 内置模块重构。
- M4 模块经济参数与业务规则调整。
- 新增 ABI 字段或治理流程变更。

## 接口/数据
### 1) 模块模板化
- 新增共享模板目录：`crates/agent_world_builtin_wasm_modules/_templates/`。
- 三类模板文件：
  - `m4_recipe_module_template.rs`
  - `m4_product_module_template.rs`
  - `m4_factory_module_template.rs`
- 各具体模块仅保留常量参数（`MODULE_ID`、配方/工厂/产品参数）并 `include!` 模板实现。

### 2) Bootstrap 描述符清单
- 在 `bootstrap_economy.rs` 新增描述符结构（最小字段）：
  - `module_id`
  - `manifest_name`
  - `max_call_rate`
- `install_m4_economy_bootstrap_modules` 按描述符循环注册/激活，不再手工逐项调用。

### 3) 一致性护栏
- 新增“描述符模块 ID 列表 == `m4_builtin_module_ids.txt` 列表”校验。
- 校验失败时返回 `WorldError::ModuleChangeInvalid`，阻断安装。

## 里程碑
- M0：设计文档 + 项目管理文档建档。
- M1：完成 M4 模块模板化重构。
- M2：完成 bootstrap 清单驱动化与一致性护栏。
- M3：回归测试与文档收口。

## 风险
- 工件哈希漂移：模板化后 wasm 二进制可能变化。
  - 缓解：执行 `scripts/sync-m4-builtin-wasm-artifacts.sh --check`，必要时同步工件清单。
- 行为回归：抽象后 reject reason 或输出格式变化。
  - 缓解：保持原判定顺序与输出字段，运行 `runtime::tests::economy_bootstrap` 与 m4 回归。
- 清单不一致：描述符与模块清单新增时漏改。
  - 缓解：引入运行时护栏与测试断言。
