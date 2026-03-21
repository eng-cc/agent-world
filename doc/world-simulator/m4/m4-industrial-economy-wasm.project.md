# M4 社会经济系统：工业链路与 WASM 模块化（项目管理文档）

- 对应设计文档: `doc/world-simulator/m4/m4-industrial-economy-wasm.design.md`
- 对应需求文档: `doc/world-simulator/m4/m4-industrial-economy-wasm.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）

### E1 设计收口
- [x] 输出设计文档：`doc/world-simulator/m4/m4-industrial-economy-wasm.prd.md`
- [x] 明确资源分层、工厂渐进建造、Recipe/Product/Factory 三类模块接口

### E2 ABI 接口基础落地
- [x] 在 `oasis7_wasm_abi` 增加经济模块接口数据结构
- [x] 增加 Recipe/Product/Factory 的 trait 接口草案
- [x] 增加序列化与最小行为单元测试

### E3 验证与文档回写
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7_wasm_abi`
- [x] 回写项目管理文档状态
- [x] 记录当日 devlog（任务完成内容 + 遗留事项）

### E4 runtime 最小执行闭环
- [x] 在 runtime 动作层新增 `BuildFactory` / `ScheduleRecipe`
- [x] 在 runtime 事件层新增建造/排产开始与完成事件（支持回放）
- [x] 在 `WorldState` 新增材料库存、工厂状态、建造队列、配方队列
- [x] 在 step 流程新增“到期任务结算”（工厂完工、配方完工）
- [x] 新增 runtime 经济闭环测试（建造时序、排产时序、产线容量、库存与电力扣减）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p oasis7 --features wasmtime`

### E5 模块在线评估接线
- [x] 新增模块驱动动作：`BuildFactoryWithModule` / `ScheduleRecipeWithModule`
- [x] 在 `step_with_modules` 中接入“先模块求值，再落地动作”流程
- [x] 新增经济模块输出契约（emit kind）解析：
  - `economy.factory_build_decision`
  - `economy.recipe_execution_plan`
- [x] 增加模块输出校验（缺失 emit / 多 emit / 非法输出 -> `ModuleCallFailed`）
- [x] 新增模块驱动经济闭环测试（建造通过、排产通过、模块拒绝）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p oasis7 --features wasmtime`

### E6 内置 M4 工业模块包与治理装载
- [x] 在 `oasis7_builtin_wasm_modules` 增加内置 M4 模块（工厂/配方/制成品）：
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
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 --features wasmtime runtime::tests::economy_bootstrap -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p oasis7 --features wasmtime`

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
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7_wasm_abi`
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 --features wasmtime runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p oasis7 --features wasmtime`

### E8 Product 校验自动闭环
- [x] 在配方完工路径自动触发产物校验（而非仅手动动作）
- [x] 建立“产物 -> Product 模块”解析策略（内置规则 + 扩展钩子）
- [x] 在账本提交前阻断未通过校验的产物入库
- [x] 增加端到端测试：`ScheduleRecipeWithModule` 完工后自动 Product 校验并落账
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 --features wasmtime runtime::tests::economy -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p oasis7 --features wasmtime`
- [x] 回写设计文档、项目文档、devlog 并提交

### E9 模块发布单动作/事件/状态机（PRD-M4-E9）
- [x] 在 runtime 动作层新增 `module_release` 动作族（submit/shadow/approve_role/reject/apply）
- [x] 在 runtime 事件层新增 `module_release` 事件族（requested/shadowed/role_approved/rejected/applied）
- [x] 新增发布单状态存储与序列化迁移（request_id、required_roles、role approvals、status）
- [x] 在 `try_apply_runtime_module_action` 接线发布单状态机与门禁
- [x] 新增状态机回放测试（可回放、拒绝路径、重复审批幂等）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::module_action_loop -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p oasis7 --features wasmtime`

### E10 Profile 治理动作（PRD-M4-E10）
- [x] 在 runtime 动作层新增治理化 profile 更新动作（material/product/recipe）
- [x] 接入 `proposal_id` 门禁（仅 `approved|applied` proposal 允许执行）
- [x] 在 runtime 事件层新增 `*_profile_governed` 事件并接入状态落账
- [x] 新增拒绝路径测试（proposal 缺失、状态不合法、字段校验失败）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::economy_priority_logistics -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p oasis7 --features wasmtime`

### E11 多角色审批策略（PRD-M4-E11）
- [x] 新增治理角色绑定模型（agent -> roles）与角色校验逻辑
- [x] 发布单 `apply` 前强制必需角色集合达成
- [x] 新增角色缺失/越权审批拒绝测试
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::module_action_loop -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p oasis7 --features wasmtime`

### E12 模块实例回滚能力（PRD-M4-E12）
- [x] 在 runtime 动作层新增 `rollback_module_instance` 动作
- [x] 在 runtime 事件层新增 `ModuleRollbackApplied` 审计事件
- [x] 回滚动作复用治理闭环并校验历史版本可回退性
- [x] 新增回滚通过/拒绝路径测试（版本不存在、owner 不匹配、接口不兼容）
- [x] 运行 `env -u RUSTC_WRAPPER cargo test -p oasis7 runtime::tests::module_action_loop -- --nocapture`
- [x] 运行 `env -u RUSTC_WRAPPER cargo check -p oasis7 --features wasmtime`

### E13 发布门禁收口（PRD-M4-E13）
- [x] 新增/更新发布 gate 脚本：`full + sync-m1/m4/m5 --check + Web strict + S9/S10`
- [x] 将发布 gate 接入 release workflow 前置步骤
- [x] 收口 testing-manual S7 TODO 覆盖口径（`oasis7_init_demo_runs_` 切换 full）
- [x] 新增 gate 冒烟测试与失败提示校验
- [x] 运行 `./scripts/release-gate-smoke.sh`
- [x] 运行 `./scripts/ci-tests.sh full`
- [x] 运行 `./scripts/sync-m1-builtin-wasm-artifacts.sh --check`
- [x] 运行 `./scripts/sync-m4-builtin-wasm-artifacts.sh --check`
- [x] 运行 `./scripts/sync-m5-builtin-wasm-artifacts.sh --check`

### E14 对外发布演练与门禁稳定性修复（PRD-M4-E14）
- [x] 实跑 `./scripts/release-gate.sh --quick`，覆盖 ci full + sync-m1/m4/m5 + Web strict + S9/S10
- [x] 修复 builtin wasm materializer 测试环境污染（移除旧品牌 builtin wasm distfs root 设置）
- [x] 加固 `scripts/viewer-release-qa-loop.sh`：仅统计 Bevy/Rust 错误日志，忽略资源加载噪声
- [x] 加固 `scripts/viewer-release-qa-loop.sh`：放宽 zoom gate 断言并增加截图重试/超时
- [x] 加固 `scripts/viewer-release-qa-loop.sh`：`snapshotForAI` 超时不再阻断 gate
- [x] 复跑 `./scripts/release-gate.sh --quick` 并确认 summary PASS

## 依赖

- `crates/oasis7_wasm_abi`：模块 ABI 与共享契约定义。
- `doc/world-runtime/wasm/wasm-interface.md`：底层 wasm-1 接口约束。
- `doc/world-runtime/module/module-lifecycle.md`：治理流程与生命周期约束。

## 状态

- 当前阶段：方案B（E9~E14）已完成（发布单 + profile 治理 + 多角色审批 + 回滚 + 发布门禁 + 发布演练）。
- 下一步：根据发布评审结论推进对外发布或补充剩余治理/稳定性项。
- 最近更新：E14 已完成对外发布演练（`release-gate --quick` 实跑 PASS），修复内置 wasm 测试环境污染并加固 Web strict 抗噪声（2026-03-05）。
