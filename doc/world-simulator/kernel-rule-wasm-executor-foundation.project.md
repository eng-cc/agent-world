# Agent World Simulator：规则 Wasm 执行接线基础（第三阶段）项目管理文档

## 任务拆解
- [x] KWE-0 输出设计文档（`doc/world-simulator/kernel-rule-wasm-executor-foundation.md`）与项目管理文档（本文件）。
- [ ] KWE-1 新增规则 wasm 输入/输出契约与可选 pre-action 评估入口。
- [ ] KWE-2 补充 wasm 接线测试（allow/deny/modify/错误兜底）。
- [ ] KWE-3 回归验证、文档与 devlog 回写。

## 依赖
- `crates/agent_world/src/simulator/kernel/mod.rs`
- `crates/agent_world/src/simulator/kernel/step.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/tests/`
- `doc/world-simulator/kernel-rule-wasm-executor-foundation.md`

## 状态
- 当前阶段：KWE-1（待开始）
- 最近更新：初始化第三阶段设计与任务拆解（2026-02-12）。
- 下一步：实现规则 wasm 输入输出契约与 pre-action 可选评估入口。
