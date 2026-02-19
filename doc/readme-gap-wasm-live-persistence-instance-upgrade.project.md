# README WASM 主链路收口：Live 模块执行 + 默认持久化模块仓库 + 模块实例化 + 升级动作（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-gap-wasm-live-persistence-instance-upgrade.md`）与项目管理文档（本文件）
- [x] T1：live/bridge 主循环切到 `step_with_modules`，并补一条 required-tier 端到端用例
- [x] T2：`save_to_dir/load_from_dir` 默认升级为包含 module store（兼容旧目录）
- [x] T3：落地模块实例化模型（`instance_id + owner + target`），替代 `module_id` 全局单实例
- [ ] T4：新增对外 `upgrade_module` 动作，要求仅接口兼容可升级，并补齐治理/审计/测试

## 依赖
- Runtime world 执行与持久化：
  - `crates/agent_world/src/runtime/world/step.rs`
  - `crates/agent_world/src/runtime/world/persistence.rs`
  - `crates/agent_world/src/runtime/world/module_actions.rs`
  - `crates/agent_world/src/runtime/world/module_tick_runtime.rs`
  - `crates/agent_world/src/runtime/state.rs`
  - `crates/agent_world/src/runtime/events.rs`
- Live bridge：
  - `crates/agent_world/src/bin/world_viewer_live/execution_bridge.rs`
  - `crates/agent_world/src/bin/world_viewer_live.rs`
- 测试：
  - `crates/agent_world/src/runtime/tests/*`
  - `crates/agent_world/src/bin/world_viewer_live/execution_bridge.rs` 内测试

## 状态
- 当前阶段：T4 待实现
- 最近更新：完成 T3，模块安装事件与运行态引入 `instance_id`，模块 action/event/tick 路由按实例执行并支持同 `module_id` 多实例并行。
- 阻塞项：无。
