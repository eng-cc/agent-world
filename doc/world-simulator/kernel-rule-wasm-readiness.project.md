# Agent World Simulator：规则 Wasm 化就绪（第二阶段）项目管理文档

## 任务拆解
- [x] KWR-0 输出设计文档（`doc/world-simulator/kernel-rule-wasm-readiness.md`）与项目管理文档（本文件）。
- [x] KWR-1 扩展 pre-action hook 接口，支持读取 `&WorldKernel` 上下文。
- [x] KWR-2 补充基于上下文的规则测试（时间/模型状态读取）。
- [x] KWR-3 回归验证、文档与 devlog 回写。

## 依赖
- `crates/agent_world/src/simulator/kernel/mod.rs`
- `crates/agent_world/src/simulator/kernel/step.rs`
- `crates/agent_world/src/simulator/tests/`
- `doc/world-simulator/kernel-rule-wasm-readiness.md`

## 状态
- 当前阶段：KWR（已完成）
- 最近更新：完成 KWR-3（回归验证、文档与 devlog 收口，2026-02-12）。
- 下一步：评估是否进入下一阶段（规则 wasm 执行接线与 cost 生效）。
