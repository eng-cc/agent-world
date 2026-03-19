# oasis7 Simulator：内核不变量回归与规则 Hook 基座（设计文档）

- 对应设计文档: `doc/world-simulator/kernel/kernel-rule-hook-foundation.design.md`
- 对应项目管理文档: `doc/world-simulator/kernel/kernel-rule-hook-foundation.project.md`

审计轮次: 5

## 1. Executive Summary
- 固化 `WorldKernel` 当前动作语义边界，先保证“现有行为不变”。
- 为后续规则模块化迁移提供最小接入点：`pre_action` / `post_action` 规则 Hook。
- 在不引入真实 WASM 执行器的前提下，先打通模拟器内核的规则决策骨架。

## 2. User Experience & Functionality
- In Scope：
  - 基于 `crates/agent_world/src/simulator/kernel/actions.rs` 增补动作行为回归测试，覆盖核心动作成功/拒绝路径。
  - 在 `WorldKernel::step` 增加 `pre_action` / `post_action` Hook 调用点。
  - 定义与 runtime 对齐的最小规则决策结构（`action_id/verdict/override_action/notes`，预留 `cost`）。
  - 保证默认未注册 Hook 时行为与现状一致。
- Out of Scope：
  - 动作语义迁移到真实 WASM 模块。
  - 改写已有动作业务规则（移动、采集、交易等）。
  - 引入新的资源结算语义或跨模块调度框架。

## 3. AI System Requirements (If Applicable)
- N/A: 本专题不新增 AI 专属要求。

## 4. Technical Specifications
- 新增（模拟器内核）规则决策结构：
  - `KernelRuleVerdict`：`allow | deny | modify`
  - `KernelRuleDecision`：`action_id`、`verdict`、`override_action`、`notes`、`cost`
  - `KernelRuleCost`：`ResourceKind -> i64`（本阶段仅透传/记录，不参与扣费）
- 新增 Hook 能力：
  - `add_pre_action_rule_hook(...)`
  - `add_post_action_rule_hook(...)`
  - 默认无 Hook。
- 规则合并语义（本阶段）：
  - `deny` 优先于 `modify/allow`
  - 多个 `modify` 需给出一致 `override_action`，否则拒绝
  - `modify` 缺失 `override_action` 视为拒绝

## 5. Risks & Roadmap
- M1：完成动作行为回归测试（不改语义）。
- M2：完成 `step` 中 `pre_action` / `post_action` Hook 接入与决策合并。
- M3：回归测试通过并补齐文档、devlog 与任务状态。

### Technical Risks
- Hook 引入后的执行顺序不稳定风险：通过固定注册顺序与测试覆盖保证确定性。
- 决策合并规则与未来 runtime 不一致风险：本阶段结构和优先级规则与 runtime 对齐。
- 回归测试覆盖不足风险：优先覆盖现有核心动作与关键拒绝分支，后续按动作枚举补齐。

## 6. Validation & Decision Record
- 追溯: 对应同名 `.project.md`，保持原文约束语义不变。
