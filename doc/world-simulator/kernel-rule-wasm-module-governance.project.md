# Agent World Simulator：规则 Wasm 模块装载治理（第五阶段）项目管理文档

## 任务拆解
- [x] KWM-0 输出设计文档（`doc/world-simulator/kernel-rule-wasm-module-governance.md`）与项目管理文档（本文件）。
- [ ] KWM-1 新增 wasm rule artifact 注册表与按 hash 激活 API。
- [ ] KWM-2 补充装载治理测试（missing hash / 冲突注册 / 激活成功路径）。
- [ ] KWM-3 回归验证、文档与 devlog 回写。

## 依赖
- `crates/agent_world/src/simulator/kernel/mod.rs`
- `crates/agent_world/src/simulator/kernel/step.rs`
- `crates/agent_world/src/simulator/tests/`
- `doc/world-simulator/kernel-rule-wasm-module-governance.md`

## 状态
- 当前阶段：KWM-1（待开始）
- 最近更新：初始化第五阶段设计与任务拆解（2026-02-12）。
- 下一步：实现 artifact 注册表与按 hash 激活 pre-action wasm 规则模块。
