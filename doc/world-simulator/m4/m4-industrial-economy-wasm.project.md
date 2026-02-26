# M4 社会经济系统：工业链路与 WASM 模块化（项目管理文档）

## 任务拆解

### E1 设计收口
- [x] 输出设计文档：`doc/world-simulator/m4/m4-industrial-economy-wasm.md`
- [x] 明确资源分层、工厂渐进建造、Recipe/Product/Factory 三类模块接口

### E2 ABI 接口基础落地
- [x] 在 `agent_world_wasm_abi` 增加经济模块接口数据结构
- [x] 增加 Recipe/Product/Factory 的 trait 接口草案
- [x] 增加序列化与最小行为单元测试

### E3 验证与文档回写
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world_wasm_abi`
- [x] 回写项目管理文档状态
- [x] 记录当日 devlog（任务完成内容 + 遗留事项）

### E4 runtime 最小执行闭环
- [x] 在 runtime 动作层新增 `BuildFactory` / `ScheduleRecipe`
- [x] 在 runtime 事件层新增建造/排产开始与完成事件（支持回放）
- [x] 在 `WorldState` 新增材料库存、工厂状态、建造队列、配方队列
- [x] 在 step 流程新增“到期任务结算”（工厂完工、配方完工）
- [x] 新增 runtime 经济闭环测试（建造时序、排产时序、产线容量、库存与电力扣减）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p agent_world --features wasmtime`

### E5 模块在线评估接线
- [x] 新增模块驱动动作：`BuildFactoryWithModule` / `ScheduleRecipeWithModule`
- [x] 在 `step_with_modules` 中接入“先模块求值，再落地动作”流程
- [x] 新增经济模块输出契约（emit kind）解析：
  - `economy.factory_build_decision`
  - `economy.recipe_execution_plan`
- [x] 增加模块输出校验（缺失 emit / 多 emit / 非法输出 -> `ModuleCallFailed`）
- [x] 新增模块驱动经济闭环测试（建造通过、排产通过、模块拒绝）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p agent_world --features wasmtime`

### E6 内置 M4 工业模块包与治理装载
- [x] 在 `agent_world_builtin_wasm_modules` 增加内置 M4 模块（工厂/配方/制成品）：
  - 工厂：`m4.factory.{miner,smelter,assembler}.mk1`
  - 配方：`m4.recipe.smelter.*` + `m4.recipe.assembler.*`
  - 制成品：`m4.product.*`
- [x] 在 runtime 增加 `m4` 嵌入工件注册层与内置清单
  - `runtime/m4_builtin_wasm_artifact.rs`
  - `runtime/world/artifacts/m4_builtin_module_ids.txt`
  - `runtime/world/artifacts/m4_builtin_modules/`
- [x] 新增治理安装入口：`World::install_m4_economy_bootstrap_modules`
- [x] 新增 m4 工件同步脚本与 CI 校验：
  - `scripts/sync-m4-builtin-wasm-artifacts.sh`
  - `scripts/ci-tests.sh` 增加 m4 工件一致性检查
- [x] 新增 runtime 闭环测试：
  - 清单一致性、安装幂等、停用后重激活
  - 基于 `WasmExecutor` 的“基础资源 -> 中间件 -> 终端制成品（logistics_drone）”链路
- [x] 运行 `./scripts/sync-m4-builtin-wasm-artifacts.sh --check`
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world --features wasmtime runtime::tests::economy_bootstrap -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p agent_world --features wasmtime`

### E7 Product 模块在线校验接线
- [x] 在 ABI 新增 `ProductValidationRequest` / `ProductValidationDecision`
- [x] 将 `ProductModuleApi` 统一为 `evaluate_product(req) -> ProductValidationDecision`
- [x] 在 runtime 动作层新增：
  - `ValidateProduct`
  - `ValidateProductWithModule`
- [x] 在 runtime 事件层新增：`ProductValidated`
- [x] 在 `step_with_modules` 接入 Product 模块求值与落地动作转换
- [x] 新增 Product 模块输出契约（emit kind）：`economy.product_validation`
- [x] 增加模块拒绝与非法输入校验（统一映射 `ActionRejected(RuleDenied)`）
- [x] 新增 runtime 测试：模块通过/拒绝两条路径
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world_wasm_abi`
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world --features wasmtime runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p agent_world --features wasmtime`

### E8 Product 校验自动闭环
- [x] 在配方完工路径自动触发产物校验（而非仅手动动作）
- [x] 建立“产物 -> Product 模块”解析策略（内置规则 + 扩展钩子）
- [x] 在账本提交前阻断未通过校验的产物入库
- [x] 增加端到端测试：`ScheduleRecipeWithModule` 完工后自动 Product 校验并落账
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p agent_world --features wasmtime runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p agent_world --features wasmtime`
- [x] 回写设计文档、项目文档、devlog 并提交

## 依赖

- `crates/agent_world_wasm_abi`：模块 ABI 与共享契约定义。
- `doc/world-runtime/wasm-interface.md`：底层 wasm-1 接口约束。
- `doc/world-runtime/module-lifecycle.md`：治理流程与生命周期约束。

## 状态

- 当前阶段：E8 完成（Product 校验自动闭环 + 入账门禁 + 端到端回归通过）。
- 下一步：M4-E5（玩家/AI 自定义模块治理模板与扩展接口）按主线节奏推进。
- 最近更新：修正文档中的内置模块 crate 名为 `agent_world_builtin_wasm_modules`，并同步校验命令为 `sync-m4-builtin-wasm-artifacts.sh --check`（2026-02-26）。
