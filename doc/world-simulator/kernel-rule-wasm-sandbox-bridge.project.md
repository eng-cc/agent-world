# Agent World Simulator：规则 Wasm Sandbox 桥接（第四阶段）项目管理文档

## 任务拆解
- [x] KWS-0 输出设计文档（`doc/world-simulator/kernel-rule-wasm-sandbox-bridge.md`）与项目管理文档（本文件）。
- [x] KWS-1 新增基于 `ModuleSandbox` 的 pre-action wasm 桥接 API 与输入/输出转换。
- [x] KWS-2 补充 sandbox 桥接测试（请求编码、allow/deny/modify、失败兜底）。
- [x] KWS-3 回归验证、文档与 devlog 回写。

## 依赖
- `crates/agent_world/src/simulator/kernel/mod.rs`
- `crates/agent_world/src/simulator/kernel/step.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/tests/`
- `crates/agent_world/src/runtime/sandbox.rs`
- `doc/world-simulator/kernel-rule-wasm-sandbox-bridge.md`

## 状态
- 当前阶段：KWS（已完成）
- 最近更新：完成 KWS-3（回归验证、文档与 devlog 收口，2026-02-12）。
- 下一步：评估下一阶段（真实 wasm 规则模块装载治理 / `KernelRuleCost` 扣费落账）。
