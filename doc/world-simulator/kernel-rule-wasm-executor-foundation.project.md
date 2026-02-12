# Agent World Simulator：规则 Wasm 执行接线基础（第三阶段）项目管理文档

## 任务拆解
- [x] KWE-0 输出设计文档（`doc/world-simulator/kernel-rule-wasm-executor-foundation.md`）与项目管理文档（本文件）。
- [x] KWE-1 新增规则 wasm 输入/输出契约与可选 pre-action 评估入口。
- [x] KWE-2 补充 wasm 接线测试（allow/deny/modify/错误兜底）。
- [x] KWE-3 回归验证、文档与 devlog 回写。

## 依赖
- `crates/agent_world/src/simulator/kernel/mod.rs`
- `crates/agent_world/src/simulator/kernel/step.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/tests/`
- `doc/world-simulator/kernel-rule-wasm-executor-foundation.md`

## 状态
- 当前阶段：KWE（已完成）
- 最近更新：完成 KWE-3（回归验证、文档与 devlog 收口，2026-02-12）。
- 下一步：评估下一阶段（真实 wasm sandbox 调用接入与策略资源扣费生效）。
