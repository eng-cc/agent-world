# Agent World Simulator：规则 Wasm Sandbox 桥接（第四阶段）项目管理文档

## 任务拆解
- [x] KWS-0 输出设计文档（`doc/world-simulator/kernel-rule-wasm-sandbox-bridge.md`）与项目管理文档（本文件）。
- [x] KWS-1 新增基于 `ModuleSandbox` 的 pre-action wasm 桥接 API 与输入/输出转换。
- [ ] KWS-2 补充 sandbox 桥接测试（请求编码、allow/deny/modify、失败兜底）。
- [ ] KWS-3 回归验证、文档与 devlog 回写。

## 依赖
- `crates/agent_world/src/simulator/kernel/mod.rs`
- `crates/agent_world/src/simulator/kernel/step.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/tests/`
- `crates/agent_world/src/runtime/sandbox.rs`
- `doc/world-simulator/kernel-rule-wasm-sandbox-bridge.md`

## 状态
- 当前阶段：KWS-2（进行中）
- 最近更新：完成 KWS-1（`WorldKernel` 与 `ModuleSandbox` pre-action 桥接接线，2026-02-12）。
- 下一步：补充 sandbox 桥接测试（请求编码、allow/deny/modify、失败兜底）。
