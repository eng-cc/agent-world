# README 高优先级缺口收口（三期）：世界内编译 + 共识动作载荷 + WASM 运行计费（项目管理文档）

## 任务拆解
- [x] T0：输出设计文档（`doc/readme-gap123-runtime-consensus-metering.md`）
- [x] T0：输出项目管理文档（本文件）
- [ ] T1：实现缺口 1（`CompileModuleArtifactFromSource` + 编译器 + required 测试）
- [ ] T2：实现缺口 2（共识动作载荷/签名/复制/执行 hook 贯通 + required 测试）
- [ ] T3：实现缺口 3（模块按次计费 + 可审计事件 + 余额不足拒绝 + required 测试）
- [ ] T4：回归验证（`cargo check` + 定向 required tests）并回写文档/devlog

## 依赖
- Runtime action/事件：
  - `crates/agent_world/src/runtime/events.rs`
  - `crates/agent_world/src/runtime/world/module_actions.rs`
  - `crates/agent_world/src/runtime/world/module_runtime.rs`
  - `crates/agent_world/src/runtime/world/event_processing.rs`
- Runtime 编译管线：
  - `crates/agent_world/src/runtime/builtin_wasm_materializer.rs`（参考）
  - `scripts/build-wasm-module.sh`
- Node 共识与复制：
  - `crates/agent_world_node/src/lib.rs`
  - `crates/agent_world_node/src/consensus_signature.rs`
  - `crates/agent_world_node/src/gossip_udp.rs`
  - `crates/agent_world_node/src/replication.rs`
  - `crates/agent_world_node/src/execution_hook.rs`
- Viewer execution bridge：
  - `crates/agent_world/src/bin/world_viewer_live/execution_bridge.rs`
- 测试：
  - `crates/agent_world/src/runtime/tests/module_action_loop.rs`
  - `crates/agent_world/src/runtime/tests/modules.rs`
  - `crates/agent_world_node/src/tests.rs`

## 状态
- 当前阶段：进行中（T0 已完成，T1/T2/T3/T4 待完成）
- 阻塞项：无
- 下一步：执行 T1（世界内编译闭环）
