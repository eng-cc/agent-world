# oasis7 Runtime：PoS 固定时间槽（Slot/Epoch）真实时钟驱动

- 对应设计文档: `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.design.md`
- 对应项目管理文档: `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.project.md`

审计轮次: 1

## 1. Executive Summary
- Problem Statement: 当前 `agent_world_node` 以本地循环推进 `next_slot`，slot 语义与 wall-clock 脱钩，重启、卡顿或循环频率变化会导致时间语义漂移。
- Proposed Solution: 在 PoS 引擎引入统一时钟模型，按 `genesis + slot_duration` 公式计算当前 slot，允许漏槽但禁止槽位倒退，并补齐入站时间窗口校验与可观测指标。
- Success Criteria:
  - SC-1: 同一 `now_ms/genesis/slot_duration_ms` 输入在多节点上计算出相同 `slot/epoch`。
  - SC-2: 节点在恢复后对齐到 wall-clock 当前槽位，不回补历史空槽（漏槽计数可观测）。
  - SC-3: 不接受未来槽提案与超窗口过旧槽提案，拒绝原因可观测。
  - SC-4: 重启后 `last_observed_slot` 单调不倒退。
  - SC-5: 至少覆盖 `test_tier_required` 定向回归与 `test_tier_full` 跨节点回归。

## 2. User Experience & Functionality
- User Personas:
  - 协议维护者：要求 slot 语义稳定且可推导。
  - 节点运营者：要求节点重启/抖动后行为可预测，可定位漏槽原因。
  - 质量复核者：要求可复现实验输入并验证时序一致性。
- User Scenarios & Frequency:
  - 每次共识参数变更后执行时序一致性回归。
  - 每次发布前执行跨节点固定时钟槽位一致性验证。
  - 节点故障恢复演练时验证漏槽统计与窗口拒绝行为。
- User Stories:
  - PRD-P2P-NODE-CLOCK-001: As a 协议维护者, I want slot to be computed from wall-clock, so that consensus timing semantics are deterministic.
  - PRD-P2P-NODE-CLOCK-002: As a 节点运营者, I want missed slots to be explicit metrics, so that lag/restart impact can be audited.
  - PRD-P2P-NODE-CLOCK-003: As a 质量复核者, I want time-window validation for inbound proposal/attestation, so that invalid-timing messages are rejected consistently.
- Critical User Flows:
  1. Flow-NODE-CLOCK-001: `节点启动 -> 读取时钟参数 -> 计算 current_slot -> 对齐本地 slot 游标`
  2. Flow-NODE-CLOCK-002: `循环周期内未出块 -> wall-clock 跨槽 -> 记录 missed_slot_count -> 在新槽继续提案`
  3. Flow-NODE-CLOCK-003: `接收提案/投票消息 -> 校验 slot 时间窗口 -> 通过则入引擎，失败则拒绝并记录原因`
  4. Flow-NODE-CLOCK-004: `节点重启 -> 恢复快照 -> 读取当前时间 -> 保证 slot 不倒退`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 动作行为 | 状态转换 | 计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 固定时间槽计算 | `genesis_unix_ms`、`slot_duration_ms`、`epoch_length_slots` | 每次 tick 计算 `current_slot/current_epoch` | `observed_slot` 单调递增 | `current_slot=floor((now-genesis)/slot_duration)`；`epoch=slot/epoch_length_slots` | 全节点一致执行，无角色分叉 |
| 漏槽对齐 | `next_slot`、`last_observed_slot`、`missed_slot_count` | `next_slot < current_slot` 时对齐并累计漏槽 | `aligned -> proposing` | `missed += current_slot-next_slot` | 仅本地引擎写入 |
| 提案准入窗口 | `proposal.slot`、`current_slot`、`max_past_slot_lag` | 拒绝未来槽/过旧槽提案 | `accepted/rejected` | `proposal.slot <= current_slot` 且 `proposal.slot + lag >= current_slot` | 仅 validator 提案可进入共识 |
| 投票准入窗口 | `attestation.slot`、`attestation.target_epoch`、`current_slot`、`max_past_slot_lag` | 拒绝未来槽/过旧槽投票，并校验目标 epoch 与提案槽位映射一致 | `accepted/rejected` | `attestation.slot <= current_slot` 且 `attestation.slot + lag >= current_slot` 且 `target_epoch == slot_epoch(proposal.slot)` | 仅已授权 validator 投票 |
| 入站拒绝可观测 | `inbound_rejected_*`、`last_inbound_timing_reject_reason` | 记录 proposal/attestation 时间窗口拒绝统计与最近拒绝原因 | `updated` | 按拒绝类型累加计数并覆盖最近原因文本 | 节点快照只读暴露 |
- Acceptance Criteria:
  - AC-1: `NodePosConfig` 支持固定时钟槽位参数（含默认值与校验）。
  - AC-2: 提案路径不再依赖纯 `next_slot += 1` 驱动；提案仅发生在 `next_slot <= current_slot`。
  - AC-3: 当 wall-clock 跨槽导致 `next_slot < current_slot` 时，系统记录漏槽并对齐，不补历史空块。
  - AC-4: 节点快照可观测 `last_observed_slot/missed_slot_count`，重启后保持单调。
  - AC-5: 入站 proposal/attestation 存在时间窗口校验，未来槽/超窗口过旧槽被拒绝。
  - AC-6: 回归覆盖单节点、重启恢复、跨节点消息窗口三类路径。
- Non-Goals:
  - 不在本任务实现 fork choice/finality 全流程升级。
  - 不在本任务改造手续费市场或交易排序策略。
  - 不在本任务引入 BLS 聚合签名与 slashing 经济惩罚。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 不适用。
- Evaluation Strategy: 不适用。

## 4. Technical Specifications
- Architecture Overview: 在 `agent_world_node::PosNodeEngine` 增加 wall-clock slot 观察与游标对齐逻辑；`slot/epoch` 来源统一为时间函数，`next_slot` 仅作为下一个可提案槽游标。
- Integration Points:
  - `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.project.md`
  - `crates/agent_world_node/src/types.rs`
  - `crates/agent_world_node/src/lib.rs`
  - `crates/agent_world_node/src/lib_impl_part1.rs`
  - `crates/agent_world_node/src/lib_impl_part2.rs`
  - `crates/agent_world_node/src/pos_state_store.rs`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - `slot_duration_ms == 0`：配置阶段拒绝。
  - `now_ms < genesis_unix_ms`：按配置选择拒绝或钳制为 slot=0，并记录告警。
  - 系统时钟回拨：禁止 `last_observed_slot` 倒退，保留最近有效槽位并记录错误。
  - 大跨度恢复：允许一次性跨越多个槽位，统一计入 `missed_slot_count`。
  - 入站消息槽位异常：统一拒绝并返回可观测拒绝原因。
- Non-Functional Requirements:
  - NFR-1: 相同输入参数下 slot 计算跨平台一致（Linux/macOS，x86_64/wasm32 测试口径一致）。
  - NFR-2: 重启恢复后 `last_observed_slot` 不倒退。
  - NFR-3: 漏槽统计更新开销不高于 O(1)/tick。
  - NFR-4: 相关变更不降低现有共识签名校验与执行绑定校验强度。
- Security & Privacy: 时间窗口拒绝不能绕过既有签名/validator 身份校验；不引入敏感数据采集。

## 5. Risks & Roadmap
- Phased Rollout:
  - M0: PRD 与项目任务建档（本轮）。
  - M1: 引擎接入 wall-clock slot 计算与漏槽对齐。
  - M2: 入站时间窗口校验与快照可观测字段。
  - M3: `test_tier_required` / `test_tier_full` 回归收口。
- Technical Risks:
  - 时钟异常或 NTP 抖动可能造成短时拒绝增多。
  - 历史测试若隐式依赖“每 tick 必提案”语义，需要同步迁移测试基线。
  - 配置默认值若与现网预期不一致，可能引入节奏变化。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-NODE-CLOCK-001 | TASK-P2P-008-T1 | `test_tier_required` | 单元测试：wall-clock slot 计算、漏槽对齐、单调性 | `agent_world_node` 共识主循环 |
| PRD-P2P-NODE-CLOCK-002 | TASK-P2P-008-T2 | `test_tier_required` | 状态快照与重启恢复测试 | 快照持久化与观测接口 |
| PRD-P2P-NODE-CLOCK-003 | TASK-P2P-008-T2/T3 | `test_tier_required` + `test_tier_full` | 跨节点 proposal/attestation 时间窗口拒绝回归 | gossip/network 入站处理链路 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-NODE-CLOCK-001 | 采用 wall-clock 公式驱动 slot | 维持 `next_slot += 1` | 后者对重启与负载变化敏感，时间语义漂移不可审计。 |
| DEC-NODE-CLOCK-002 | 允许漏槽并显式统计 | 补写历史空槽 | 补写会引入伪历史并增加状态复杂度。 |
| DEC-NODE-CLOCK-003 | 时间窗口拒绝与签名校验并行生效 | 仅签名校验不校验时间 | 仅签名校验无法阻断未来槽/陈旧槽重放噪音。 |
