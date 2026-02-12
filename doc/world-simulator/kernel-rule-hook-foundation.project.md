# Agent World Simulator：内核不变量回归与规则 Hook 基座（项目管理文档）

## 任务拆解
- [x] KRH-1 补充内核动作行为回归测试（以当前行为为基线，不改语义）。
- [x] KRH-2 在 `WorldKernel::step` 接入 `pre_action` / `post_action` Hook（默认 no-op）。
- [x] KRH-3 定义模拟器规则决策结构并实现决策合并。
- [x] KRH-4 回归验证、文档与 devlog 回写。

## 依赖
- `crates/agent_world/src/simulator/kernel/actions.rs`
- `crates/agent_world/src/simulator/kernel/step.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/kernel/mod.rs`
- `crates/agent_world/src/simulator/tests/`

## 状态
- 当前阶段：KRH-4（已完成）
- 最近更新：完成 KRH-4（回归验证与文档收口）：
  - 执行并通过全量回归：`env -u RUSTC_WRAPPER cargo test -p agent_world`（2026-02-12）。
  - KRH 任务状态全部完成（KRH-1~KRH-4）。
  - 同步回写项目管理文档与 devlog。
- 下一步：KRH 基座阶段结束，是否继续下一阶段（例如规则 `cost` 生效与 wasm 规则执行器接线）待确认。
