# p2p PRD

审计轮次: 5

## 目标
- 建立 p2p 模块设计主文档，统一需求边界、技术方案与验收标准。
- 确保 p2p 模块后续改动可追溯到 PRD-ID、任务和测试。

## 范围
- 覆盖 p2p 模块当前能力设计、接口边界、测试口径与演进路线。
- 覆盖 PRD-ID 到 `doc/p2p/prd.project.md` 的任务映射。
- 不覆盖实现代码逐行说明与历史过程记录。

## 接口 / 数据
- PRD 主入口: `doc/p2p/prd.md`
- 项目管理入口: `doc/p2p/prd.project.md`
- 文件级索引: doc/p2p/prd.index.md
- 追踪主键: `PRD-P2P-xxx`
- 测试与发布参考: `testing-manual.md`

## 里程碑
- M1 (2026-03-03): 完成模块设计 PRD 主体重写与任务改造。
- M2: 补齐模块设计验收清单与关键指标。
- M3: 建立 PRD-ID -> Task -> Test 的长期追踪闭环。

## 风险
- 模块边界演进快，文档同步可能滞后。
- 指标口径不稳定会降低验收一致性。
## 1. Executive Summary
- Problem Statement: 网络、共识、DistFS 与节点激励相关设计迭代频繁，缺少统一 PRD 导致跨子系统改动难以同时满足可用性、安全性与可审计性。
- Proposed Solution: 以 p2p PRD 统一定义分布式系统的目标拓扑、共识约束、存储策略、奖励机制与发布门禁。
- Success Criteria:
  - SC-1: P2P 关键改动 100% 映射到 PRD-P2P-ID。
  - SC-2: 多节点在线长跑套件按计划执行并形成可追溯结果。
  - SC-3: 共识与存储链路关键失败模式具备回归测试覆盖。
  - SC-4: 发行前完成网络/共识/DistFS 三线联合验收。
  - SC-5: 移动端轻客户端路径可在不运行本地权威模拟器前提下稳定接入。
  - SC-6: PoS slot/epoch 在多节点间由统一时间公式驱动，允许漏槽但不出现时间语义倒退。
  - SC-7: PoS 支持槽内 logical tick 相位门控与动态节拍调度，实现可配置 `tick/slot` 语义。

## 2. User Experience & Functionality
- User Personas:
  - 协议工程师：需要明确网络与共识边界。
  - 节点运营者：需要稳定部署和可观测运行信号。
  - 安全评审者：需要签名、治理、资产流转的可审计证据。
  - 移动端玩家：需要低算力设备可持续在线并获得正确最终性反馈。
- User Scenarios & Frequency:
  - 协议演进评审：每次共识或网络协议改动前执行。
  - 多节点长跑：按周执行并记录稳定性与恢复结果。
  - 发行前联合验收：每个候选版本执行一次三线联测。
  - 安全审计复核：关键资产链路改动后立即触发。
  - 轻客户端接入验收：每次移动端协议调整后执行输入/最终性/重连验证。
- User Stories:
  - PRD-P2P-001: As a 协议工程师, I want explicit protocol boundaries, so that multi-crate changes remain coherent.
  - PRD-P2P-002: As a 节点运营者, I want reliable longrun validation, so that production confidence increases.
  - PRD-P2P-003: As a 安全评审者, I want auditable cryptographic and governance flows, so that risk is controlled.
  - PRD-P2P-004: As a 移动端玩家, I want intent-only light client access, so that low-end devices can still participate fairly.
  - PRD-P2P-005: As a 协议工程师, I want slot/epoch to be wall-clock driven, so that block time semantics remain stable across restart and lag.
  - PRD-P2P-006: As a 协议工程师, I want slot-internal tick-phase pacing, so that proposal cadence can follow configured `ticks_per_slot`.
- Critical User Flows:
  1. Flow-P2P-001: `网络拓扑变更 -> 共识联调 -> DistFS 同步 -> 节点状态一致性验证`
  2. Flow-P2P-002: `执行 S9/S10 长跑 -> 采集故障与恢复数据 -> 输出收敛报告`
  3. Flow-P2P-003: `资产/签名链路变更 -> 审计检查 -> 安全门禁 -> 发布判定`
  4. Flow-P2P-004: `手机端提交签名 intent -> 权威模拟执行 -> 链上承诺/挑战 -> 客户端 final 确认`
  5. Flow-P2P-005: `节点读取 wall-clock -> 计算 slot/epoch -> 允许漏槽推进 -> 拒绝未来槽/过旧槽提案`
  6. Flow-P2P-006: `节点按 wall-clock 计算 logical tick/phase -> 相位命中才提案 -> runtime 动态等待下一 tick 边界`
- Functional Specification Matrix:
| 功能点 | 字段定义 | 按钮/动作行为 | 状态转换 | 排序/计算规则 | 权限逻辑 |
| --- | --- | --- | --- | --- | --- |
| 网络与共识协同 | 节点ID、轮次、提交高度、延迟 | 启动联测并比对共识结果 | `joining -> syncing -> committed` | 高度/轮次单调递增 | 仅授权节点参与共识 |
| DistFS 复制 | 文件ID、副本状态、同步延迟 | 触发复制并校验完整性 | `queued -> replicating -> verified` | 优先关键数据副本 | 节点需满足存储策略 |
| 长跑与恢复 | 失败类型、恢复动作、恢复时长 | 注入故障并执行恢复流程 | `stable -> degraded -> recovered` | 按故障等级排序处理 | 运维/评审可操作恢复流程 |
| 轻客户端权威状态 | `intent(tick/seq/sig)`、`state_root`、`finality_state` | 手机端只上报 intent，接收 delta/proof 并展示最终性 | `pending -> confirmed -> final` | 按 tick 排序，重复 seq 幂等去重 | 权威状态仅由模拟节点提交，客户端无写权限 |
| PoS 固定时间槽 | `genesis_unix_ms`、`slot_duration_ms`、`epoch_length_slots`、`last_observed_slot`、`missed_slot_count` | 每次 tick 按真实时间换算 slot；仅在 `next_slot <= current_slot` 时允许提案 | `pending -> committed/rejected`（槽位单调） | `current_slot=floor((now-genesis)/slot_duration)`；`epoch=slot/epoch_length_slots` | 仅验证者可提案/投票；未来槽消息拒绝 |
| PoS 槽内 tick 节拍 | `ticks_per_slot`、`tick_phase`、`proposal_tick_phase`、`last_observed_tick`、`missed_tick_count` | 仅在命中提案相位时触发提案；worker 按下一 logical tick 边界动态调度 | `idle -> proposing`（相位门控） | `logical_tick=floor((now-genesis)*ticks_per_slot/slot_duration)`；`phase=tick%ticks_per_slot` | 节拍公式全节点一致；本地调度可回退固定间隔 |
- Acceptance Criteria:
  - AC-1: p2p PRD 覆盖网络、共识、存储、激励四条主线。
  - AC-2: p2p project 文档任务项明确映射 PRD-P2P-ID。
  - AC-3: 与 `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md` 等设计文档口径一致。
  - AC-4: S9/S10 相关测试套件在 testing 手册中有对应条目。
  - AC-5: 轻客户端专题需求落盘并映射到独立任务链（`TASK-P2P-MLC-*`）。
  - AC-6: `node-pos-slot-clock-real-time-2026-03-07` 专题文档落盘并映射任务链 `TASK-P2P-008`。
  - AC-7: `node-pos-subslot-tick-pacing-2026-03-07` 专题文档落盘并映射任务链 `TASK-P2P-009`。
- Non-Goals:
  - 不在本 PRD 细化 viewer UI 交互。
  - 不替代 runtime 内核的模块执行细节设计。

## 3. AI System Requirements (If Applicable)
- Tool Requirements: 长跑脚本、链路探针、反馈注入、共识日志分析工具。
- Evaluation Strategy: 以在线稳定时长、分叉恢复成功率、反馈链路可用性、错误收敛时间评估。

## 4. Technical Specifications
- Architecture Overview: p2p 模块负责 `agent_world_net`/`agent_world_consensus`/`agent_world_distfs` 与 node 侧分布式运行协同，强调一致性与故障恢复。
- Integration Points:
  - `doc/p2p/blockchain/production-grade-blockchain-p2pfs-roadmap.prd.md`
  - `doc/p2p/distributed/distributed-hard-split-phase7.prd.md`
  - `doc/p2p/network/p2p-mobile-light-client-authoritative-state-2026-03-06.prd.md`
  - `doc/p2p/node/node-pos-slot-clock-real-time-2026-03-07.prd.md`
  - `doc/p2p/node/node-pos-subslot-tick-pacing-2026-03-07.prd.md`
  - `doc/p2p/token/mainchain-token-allocation-mechanism-phase2-governance-bridge-distribution-2026-02-26.prd.md`
  - `testing-manual.md`
- Edge Cases & Error Handling:
  - 节点掉线：共识链路需在节点恢复后自动重同步并验证状态。
  - 网络分区：检测分区后阻断不安全提交并等待合并恢复。
  - 轻客户端弱网：启用低频增量+关键帧同步并保持最终性状态不倒退。
  - 空副本：DistFS 副本不足时触发补副本任务并记录告警。
  - 超时：共识轮次超时后执行回退/重试策略。
  - 并发冲突：同高度多提交候选按共识规则拒绝冲突分支。
  - 数据损坏：校验失败副本立即隔离并重建。
  - 时钟回拨/漂移：wall-clock 出现回拨时禁止 slot 倒退；超阈值漂移进入拒绝或告警路径。
  - 大跨度漏槽：节点恢复后按当前 wall-clock 对齐 slot，并累加漏槽计数，不补历史空块。
- Non-Functional Requirements:
  - NFR-P2P-1: 多节点长跑稳定性指标持续达标并可追溯。
  - NFR-P2P-2: 共识提交与复制链路关键失败模式覆盖率 100%。
  - NFR-P2P-3: 节点异常恢复流程具备标准化操作与证据产物。
  - NFR-P2P-4: 资产与签名链路审计记录完整率 100%。
  - NFR-P2P-5: 协议演进不得破坏既有网络兼容性基线。
  - NFR-P2P-6: 手机轻客户端路径必须可验证最终性，且不要求端侧权威模拟。
  - NFR-P2P-7: slot 计算在重启前后保持单调一致；槽位倒退容忍度为 0（仅允许漏槽）。
  - NFR-P2P-8: 在启用 `ticks_per_slot` 时，logical tick/phase 计算跨节点一致，提案节拍可观测且可回归验证。
- Security & Privacy: 需保证节点身份、签名、账本与反馈数据链路的完整性；所有关键动作必须具备可审计记录。

## 5. Risks & Roadmap
- Phased Rollout:
  - MVP (2026-03-03): 固化网络/共识/存储统一设计基线。
  - v1.1: 补齐在线长跑失败模式和恢复手册。
  - v2.0: 建立分布式质量趋势看板（稳定性、时延、恢复、失败率）。
- Technical Risks:
  - 风险-1: 多子系统并行演进带来接口漂移。
  - 风险-2: 长跑测试覆盖不足导致线上异常暴露滞后。

## 6. Validation & Decision Record
- Test Plan & Traceability:
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-P2P-001 | TASK-P2P-001/002/005 | `test_tier_required` | 网络/共识/存储联合验收清单检查 | 协议边界与跨 crate 兼容 |
| PRD-P2P-002 | TASK-P2P-002/003/005 | `test_tier_required` + `test_tier_full` | S9/S10 长跑与恢复演练 | 多节点稳定性与故障恢复 |
| PRD-P2P-003 | TASK-P2P-003/004/005 | `test_tier_full` | 签名与治理链路审计检查 | 资产安全与发布风险控制 |
| PRD-P2P-004 | TASK-P2P-006/007 | `test_tier_required` + `test_tier_full` | 轻客户端 intent/finality/challenge/reconnect 闭环验证 | 移动端接入、公平性与可用性 |
| PRD-P2P-005 | TASK-P2P-008 | `test_tier_required` + `test_tier_full` | 固定时间槽单调性/漏槽/重启恢复/未来槽拒绝回归 | 共识时间语义、提案与投票窗口 |
| PRD-P2P-006 | TASK-P2P-009 | `test_tier_required` + `test_tier_full` | 槽内 tick 相位门控、动态调度等待与跨节点节拍回归 | 共识提案节奏、runtime 调度与可观测 |
- Decision Log:
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-P2P-001 | 网络/共识/DistFS 统一验收 | 子系统独立验收 | 可降低跨链路隐性回归风险。 |
| DEC-P2P-002 | 长跑结果进入发布门禁 | 仅开发阶段抽样运行 | 发布质量依赖真实长稳证据。 |
| DEC-P2P-003 | 关键动作全链路审计 | 仅关键节点日志 | 审计深度不足会放大安全风险。 |
| DEC-P2P-004 | 移动端采用轻客户端+链下权威模拟 | 手机端参与权威模拟 | 移动端资源受限，权威性和实时性需分层保障。 |
| DEC-P2P-005 | PoS slot 按 wall-clock 统一公式驱动 | 继续本地 tick 自增 slot | 可消除重启/负载抖动造成的时间语义漂移。 |
| DEC-P2P-006 | PoS 增加槽内 tick 相位门控与动态调度 | 仅保留固定 `tick_interval` 与 slot 门控 | 需要稳定落地 `10 tick/slot` 节奏并降低固定 sleep 漂移。 |
