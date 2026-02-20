# README WASM 主链路收口：Live 模块执行 + 默认持久化模块仓库 + 模块实例化 + 升级动作（设计文档）

## 目标
- 收口 README 中 WASM 主链路承诺与工程实现差距，覆盖以下四项：
  - live/bridge 主循环切换到 `step_with_modules`（或等价模块执行管线）。
  - `save_to_dir/load_from_dir` 默认包含 module store 闭环，避免恢复后缺失 artifact bytes。
  - 落地模块实例化模型：`instance_id + owner + target`，替代 `module_id` 全局单实例覆盖语义。
  - 新增对外 `upgrade_module` 动作，并要求“仅接口兼容”时才允许升级。
- 保持 runtime/consensus/audit 的可追溯性，不破坏现有 required-tier 主路径。

## 范围
- In scope
  - `crates/agent_world/src/bin/world_viewer_live/execution_bridge.rs`
  - `crates/agent_world/src/bin/world_viewer_live.rs`
  - `crates/agent_world/src/runtime/world/persistence.rs`
  - `crates/agent_world/src/runtime/events.rs`
  - `crates/agent_world/src/runtime/state.rs`
  - `crates/agent_world/src/runtime/world/module_actions.rs`
  - `crates/agent_world/src/runtime/world/module_tick_runtime.rs`
  - `crates/agent_world/src/runtime/tests/*` 与 `crates/agent_world/src/bin/world_viewer_live/execution_bridge.rs` 内测试
- Out of scope
  - 大规模重写 `agent_world_wasm_abi` 协议结构（尽量通过 runtime 层扩展实现）。
  - 浏览器 wasm32 节点网络能力增强。
  - 变更 third_party 代码。

## 接口 / 数据
### 1) Live/Bridge WASM 执行链路
- `NodeRuntimeExecutionDriver` 持有可复用 `WasmExecutor`。
- committed action 回放后执行 `step_with_modules(&mut sandbox)`，保证 action/event/tick 模块路由生效。
- reward runtime 的 execution bridge 同步路径采用同口径模块执行。

### 2) 默认持久化包含 module store
- `World::save_to_dir` 默认调用 module store 保存（registry + meta + artifact）。
- `World::load_from_dir` 默认尝试 module store 恢复 artifact bytes；兼容旧目录（无 module store）时保持可加载。
- `save_to_dir_with_modules/load_from_dir_with_modules` 保留为兼容入口，可委托默认实现。

### 3) 模块实例化模型
- 新增运行时实例状态（示意）：
  - `module_instance_id -> { module_id, module_version, wasm_hash, owner_agent_id, install_target, active, installed_at }`
- `ModuleInstalled` 事件补充 `instance_id`（兼容老事件默认值）。
- tick 路由按实例执行，而非按 `module_id` 单键覆盖。

### 4) 升级动作与兼容性校验
- 新增动作（示意）：`UpgradeModuleFromArtifact { upgrader_agent_id, instance_id, next_manifest, activate }`。
- 新增领域事件（示意）：`ModuleUpgraded { instance_id, from_module_version, to_module_version, ... }`。
- 升级前校验：
  - 升级发起者必须匹配实例 owner。
  - `next_manifest.module_id` 必须与实例 `module_id` 一致。
  - 仅接口兼容允许升级：
    - `interface_version` 不变；
    - 必需导出（entrypoint）不减少；
    - 既有订阅 stage/filter 不被破坏（至少旧订阅可被覆盖）；
    - ABI 契约关键字段（input/output schema 成对约束、cap slot 引用合法性）保持兼容。

## 里程碑
- M1：T0 文档与任务拆解完成。
- M2：T1 live/bridge 主循环切换 + required-tier e2e 用例完成。
- M3：T2 默认持久化 module store 闭环完成。
- M4：T3 模块实例化模型落地并覆盖核心测试。
- M5：T4 升级动作 + 兼容性校验 + 测试收口。

## 风险
- 行为变更风险：live 入口切换到 `step_with_modules` 可能暴露历史未触发的模块副作用。
- 兼容性风险：实例化模型引入新字段后需保证旧快照/旧事件反序列化兼容。
- 性能风险：按实例 tick 路由会增加调用次数，需要通过排序/去重保持确定性与可控成本。
- 语义风险：接口兼容判定过严会阻塞升级，过松会引入运行时崩溃；需配套拒绝路径测试。
