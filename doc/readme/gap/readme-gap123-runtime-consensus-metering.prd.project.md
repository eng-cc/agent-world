# README 高优先级缺口收口（三期）：世界内编译 + 共识动作载荷 + WASM 运行计费（项目管理文档）

审计轮次: 3

## 审计备注
- 主项目入口：`doc/readme/gap/readme-gap-distributed-prod-hardening-gap12345.prd.project.md`。
- 本文件仅维护本专题增量任务。

## 任务拆解
- [x] T0：输出设计文档（`doc/readme/gap/readme-gap123-runtime-consensus-metering.prd.md`）
- [x] T0：输出项目管理文档（本文件）
- [x] T1：实现缺口 1（`CompileModuleArtifactFromSource` + 编译器 + required 测试）
- [x] T2：实现缺口 2（共识动作载荷/签名/复制/执行 hook 贯通 + required 测试）
- [x] T3：实现缺口 3（模块按次计费 + 可审计事件 + 余额不足拒绝 + required 测试）
- [x] T4：回归验证（`cargo check` + 定向 required tests）并回写文档/devlog

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
- 当前阶段：已完成（T0~T4 全部完成）
- 阻塞项：无
- 下一步：无（本轮 README 缺口 1/2/3 收口完成）

## 迁移记录（2026-03-03）
- 已按 `TASK-ENGINEERING-014-D1 (PRD-ENGINEERING-006)` 从 legacy 命名迁移为 `.prd.md/.prd.project.md`。
- 保留原任务拆解、依赖与状态语义，不改变既有结论。
