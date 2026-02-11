# Agent World Simulator：内核不变量回归与规则 Hook 基座（项目管理文档）

## 任务拆解
- [x] KRH-1 补充内核动作行为回归测试（以当前行为为基线，不改语义）。
- [ ] KRH-2 在 `WorldKernel::step` 接入 `pre_action` / `post_action` Hook（默认 no-op）。
- [ ] KRH-3 定义模拟器规则决策结构并实现决策合并。
- [ ] KRH-4 回归验证、文档与 devlog 回写。

## 依赖
- `crates/agent_world/src/simulator/kernel/actions.rs`
- `crates/agent_world/src/simulator/kernel/step.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/kernel/mod.rs`
- `crates/agent_world/src/simulator/tests/`

## 状态
- 当前阶段：KRH-2（进行中）
- 最近更新：完成 KRH-1（新增 `kernel_action_behavior_snapshot_stays_stable` 基线回归测试），并通过 `env -u RUSTC_WRAPPER cargo test -p agent_world kernel_action_behavior_snapshot_stays_stable -- --nocapture`（2026-02-12）。
- 下一步：推进 KRH-2，在 `WorldKernel::step` 接入 pre/post Hook（默认 no-op，不改变行为）。
