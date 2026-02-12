# Agent World Simulator：内核不变量回归与规则 Hook 基座（项目管理文档）

## 任务拆解
- [x] KRH-1 补充内核动作行为回归测试（以当前行为为基线，不改语义）。
- [x] KRH-2 在 `WorldKernel::step` 接入 `pre_action` / `post_action` Hook（默认 no-op）。
- [x] KRH-3 定义模拟器规则决策结构并实现决策合并。
- [ ] KRH-4 回归验证、文档与 devlog 回写。

## 依赖
- `crates/agent_world/src/simulator/kernel/actions.rs`
- `crates/agent_world/src/simulator/kernel/step.rs`
- `crates/agent_world/src/simulator/kernel/types.rs`
- `crates/agent_world/src/simulator/kernel/mod.rs`
- `crates/agent_world/src/simulator/tests/`

## 状态
- 当前阶段：KRH-3（已完成）
- 最近更新：完成 KRH-3（新增 `KernelRuleDecision/KernelRuleVerdict/KernelRuleCost` 与 `merge_kernel_rule_decisions`，并在 `WorldKernel::step` 接入 deny/modify/allow 决策执行，`cost` 仅记录不扣费），新增规则决策回归测试：
  - `kernel_pre_action_rule_deny_rejects_action`
  - `kernel_pre_action_rule_modify_overrides_action`
  - `kernel_conflicting_modify_decisions_are_denied`
  - `merge_kernel_rule_decisions_rejects_conflicting_overrides`
  - `merge_kernel_rule_decisions_rejects_missing_override`
  - `post_action_hook_receives_event_after_modify_decision`
  - 兼容回归：`kernel_rule_hooks_default_path_keeps_action_behavior`、`kernel_rule_hooks_run_in_registration_order`、`kernel_post_action_hook_receives_emitted_event`、`kernel_action_behavior_snapshot_stays_stable`
  - 测试命令：`env -u RUSTC_WRAPPER cargo test -p agent_world kernel_rule_decisions -- --nocapture && env -u RUSTC_WRAPPER cargo test -p agent_world kernel_rule_hooks_default_path_keeps_action_behavior -- --nocapture && env -u RUSTC_WRAPPER cargo test -p agent_world kernel_rule_hooks_run_in_registration_order -- --nocapture && env -u RUSTC_WRAPPER cargo test -p agent_world kernel_post_action_hook_receives_emitted_event -- --nocapture && env -u RUSTC_WRAPPER cargo test -p agent_world kernel_action_behavior_snapshot_stays_stable -- --nocapture`（2026-02-12）。
- 下一步：推进 KRH-4（回归验证、文档与 devlog 收口）。
