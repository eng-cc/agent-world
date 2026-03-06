# Gameplay Distributed Consensus Governance Long-Run（生产级详细设计）

审计轮次: 3

- 对应项目管理文档: `doc/game/gameplay/gameplay-distributed-consensus-governance-longrun-2026-03-06.prd.project.md`

## ROUND-002 主从口径
- 主入口文档：`doc/game/gameplay/gameplay-top-level-design.prd.md`。
- 本文档聚焦“长期不下线多人模拟”的分布式执行共识、治理共识、身份与反女巫设计，不重复顶层玩法体验目标。

## 0. 设计摘要（必备字段）

### 0.1 目标
- 将现有 `Action -> DomainEvent -> State` 固化为可复制状态机（RSM）协议，满足全序、可重放、可审计。
- 将规则与参数变更统一收敛到治理事件流，并强制 `epoch` 边界生效，禁止即时改规则。
- 引入身份信誉/抵押与惩罚体系，降低多号操纵治理和经济的系统性风险。

### 0.2 范围
#### In Scope
- Tick/Block 级执行证书（`parent_hash + events_hash + state_root`）模型。
- 确定性执行约束（随机数、时钟、事件排序、版本锁）。
- 治理流 `提案 -> 投票 -> timelock -> 生效` 的状态机和时序。
- 紧急刹车、紧急否决的权限模型、触发条件和审计要求。
- 身份、信誉、抵押、惩罚与申诉流程的约束模型。

#### Out of Scope
- 共识网络传输协议细节（P2P 帧格式、gossip 拓扑、NAT 穿透）。
- 完整链上经济学参数调优（发行曲线、税率具体数值）。
- 前端治理页面交互稿和可视化设计细节。

### 0.3 接口/数据
- 运行时执行闭环：`crates/agent_world/src/runtime/world/step.rs`
- 世界状态与事件：`crates/agent_world/src/runtime/state.rs`、`crates/agent_world/src/runtime/events.rs`
- 快照/回放：`crates/agent_world/src/runtime/world/persistence.rs`、`crates/agent_world/src/runtime/snapshot.rs`
- 现有治理链路：`crates/agent_world/src/runtime/world/governance.rs`

### 0.4 里程碑
- DCG-M1: Tick 证书数据结构、哈希链与审计查询落地（write-only，不改业务规则）。
- DCG-M2: 执行确定性收口（随机信标、时钟约束、回放一致性门禁）。
- DCG-M3: 治理 timelock + epoch 生效 + 紧急控制闭环。
- DCG-M4: 身份与反女巫上线（信誉/抵押/惩罚）并完成长稳演练。

### 0.5 风险
- 证书链引入后写放大与存储成本上升。
- 治理生效延迟会降低“快速修复”体验，需要紧急机制兜底。
- 反女巫规则若过强，可能误伤新玩家参与率。

## 1. 执行共识层（世界状态）

### 1.1 现状映射
- 现有 runtime 已具备 RSM 核心形态：
  - `World::step` 以 `state.time` 为逻辑 tick 推进。
  - 事件通过 `append_event` 进入 `journal.events`，具备全局递增 `event_id`。
  - `snapshot()/from_snapshot()` 支持状态快照与日志回放恢复。
- 缺口：缺少“每个 tick 的可验证证书”和“跨节点一致执行证明”。

### 1.2 目标不变量（必须长期成立）
- INV-RSM-001：同一 `tick` 输入序列在同一版本执行器上必须产出同一 `events_hash`。
- INV-RSM-002：`state_root(t)` 只由 `state_root(t-1)` 与 `ordered_events(t)` 决定。
- INV-RSM-003：`parent_hash` 必须形成单链，禁止分叉提交到主账本。
- INV-RSM-004：本地墙钟时间不得参与状态转移决策。

### 1.3 数据模型
| 模型 | 字段 | 说明 |
| --- | --- | --- |
| `TickBlockHeader` | `epoch`, `tick`, `parent_hash`, `events_hash`, `state_root`, `executor_version`, `randomness_seed` | Tick 头，不含签名 |
| `TickBlock` | `header`, `ordered_action_ids`, `ordered_event_ids`, `event_count` | 可回放执行输入输出索引 |
| `TickCertificate` | `block_hash`, `consensus_height`, `threshold`, `signatures{node_id->sig}` | 共识签名证书 |
| `ExecutionDigest` | `action_batch_hash`, `domain_events_hash`, `state_projection_hash` | 审计与快速比对摘要 |

### 1.4 哈希与证书规范
- `events_hash = H(event_hash_1 || ... || event_hash_n)`，事件哈希使用稳定序列化（禁止 HashMap 无序迭代）。
- `state_root = H(canonical_state_projection)`，投影至少覆盖：
  - `WorldState`（可执行状态）
  - `Manifest hash`（运行规则版本）
  - `Policy set hash`（规则参数）
- `block_hash = H("tickblock:v1|parent_hash|tick|events_hash|state_root|executor_version")`
- `TickCertificate` 只对 `block_hash` 签名，验签失败的 block 不得进入可见历史。
- v1 实现约束（2026-03-06）：
  - 先使用 `sha256` 本地签章占位（`threshold=1`）打通证书链路与回放校验。
  - 后续在 `TASK-GAME-010` 前替换为多签验签路径并接入治理权限门禁。

### 1.5 执行流程（单分片）
1. Ingress：收集 `pending_actions`，封板为 tick 输入集。
2. Ordering：对输入集按共识顺序编号（`order_index`），得到全序动作列表。
3. Execute：按顺序执行 `Action -> DomainEvent -> State`。
4. Digest：计算 `events_hash/state_root/block_hash`。
5. Certify：达到阈值签名后产出 `TickCertificate`。
6. Commit：写入 `TickBlock + TickCertificate`，并可触发快照策略。

### 1.6 确定性设计（随机数与时钟）
- 随机数：
  - `epoch_seed` 由 VRF 聚合生成，按 `epoch` 固定。
  - `tick_seed = H("tick-rand:v1|epoch_seed|tick|parent_hash")`。
  - 业务逻辑只允许读取 `tick_seed` 派生随机，不得直接调用本地随机源。
- 时钟：
  - 状态机只使用 `state.time`（逻辑 tick）。
  - 客户端提交时间仅用于审计字段，不参与规则计算。

### 1.7 恢复与审计
- 恢复流程：`latest snapshot -> replay tick blocks -> verify every certificate -> compare final state_root`。
- 审计接口最小集：
  - `query_tick_block(tick) -> TickBlock + TickCertificate`
  - `query_state_root(tick) -> state_root`
  - `verify_tick_chain(from, to) -> bool + mismatch_detail`

## 2. 治理共识层（规则与参数）

### 2.1 治理对象分层
| 层级 | 示例 | 生效策略 |
| --- | --- | --- |
| 参数层 | 税率、投票门槛、危机频率 | 提案通过后 `timelock`，按 `epoch` 边界生效 |
| 规则层 | 行为校验策略、惩罚规则 | 必须走治理流，禁止热改 |
| 协议层 | 事件 schema、执行器版本 | 两阶段升级（兼容期 + 强制期） |

### 2.2 治理状态机
`Draft -> Voting -> Queued(timelock) -> Executable(epoch-boundary) -> Applied/Rejected/Expired`

新增治理事件建议：
- `GovernanceChangeProposed`
- `GovernanceVoteCast`
- `GovernanceVoteFinalized`
- `GovernanceChangeQueued`
- `GovernanceChangeApplied`
- `GovernanceChangeRejected`
- `GovernanceChangeExpired`

### 2.3 epoch 生效规则
- `epoch_len_ticks` 固定配置（可治理变更，但变更本身也需 timelock）。
- 任何提案即使通过，也只能在 `activate_epoch >= current_epoch + min_activation_delay_epochs` 时生效。
- 运行时在 `epoch` 过渡点一次性切换规则版本，避免 tick 内规则漂移。

### 2.4 timelock 与可预期性
- `timelock_ticks` 是提案元数据必填项。
- `Queued` 阶段必须公开：
  - `queued_at_tick`
  - `not_before_tick`
  - `activate_epoch`
  - `target_manifest_hash` / `target_policy_hash`
- 在 `not_before_tick` 前，任何 `apply` 请求应被拒绝并写拒绝事件。

### 2.5 紧急刹车与紧急否决（宪章约束）
#### 紧急刹车（Emergency Brake）
- 作用：短时冻结高风险状态变更（非关键经济/战争行为），保留只读和恢复操作。
- 触发门槛：`guardian_threshold` 多签 + 审计理由必填。
- 自动失效：最长持续 `max_brake_epochs`，到期未续签自动解除。

#### 紧急否决（Emergency Veto）
- 作用：在提案 `Queued` 阶段撤销明显恶意或重大漏洞提案。
- 触发门槛：高于普通通过门槛的超级多数。
- 审计要求：必须记录证据 hash、发起人、签名集合和申诉窗口。

#### 宪章要求
- 权限主体、阈值、有效窗口、可触发场景写入治理宪章（建议后续形成独立 PRD）。
- 任何越权触发必须自动拒绝并落审计事件。

### 2.6 与现有治理代码的迁移关系
- 现有 `propose -> shadow -> approve -> apply` 可映射为：
  - `Proposed/Shadowed` 对应 `Draft` 阶段预校验。
  - `Approved` 对应 `Voting` 完成。
  - 新增 `Queued + epoch gate` 才允许 `Applied`。
- `GovernanceFinalityCertificate` 继续复用，但签名载荷需补充 `activate_epoch` 与 `timelock` 字段。

## 3. 身份与反女巫

### 3.1 身份模型
| 模型 | 字段 | 说明 |
| --- | --- | --- |
| `PlayerIdentity` | `player_id`, `pubkey`, `created_tick`, `reputation_score`, `stake_locked`, `status` | 玩家治理/经济身份 |
| `ValidatorIdentity` | `node_id`, `pubkey`, `stake_locked`, `uptime_score`, `slash_count` | 共识签名节点身份 |
| `IdentityLinkEvidence` | `source_id`, `target_id`, `evidence_kind`, `confidence`, `observed_tick` | 反女巫关联证据 |

### 3.2 权重与冷启动约束
- 投票权重建议：
  - `vote_weight = min(cap, sqrt(stake_locked)) * reputation_multiplier * activity_multiplier`
- 新身份冷启动：
  - `warmup_epochs` 内权重上限受限，防止短时批量建号投票。
- 快照机制：
  - 投票权重按提案开启时快照，避免“闪电抵押-投票-撤押”。

### 3.3 反女巫检测与处置
- 检测输入：
  - 资金同源图谱、行为高度同步、委托网络异常聚类、短时新号爆发。
- 风险处置：
  - 低风险：权重衰减与人工复核。
  - 中风险：临时冻结治理权、限制经济高价值动作。
  - 高风险：触发惩罚流程（降权/冻结/驱逐/罚没）。

### 3.4 惩罚矩阵
| 违规类型 | 触发证据 | 惩罚 | 恢复条件 |
| --- | --- | --- | --- |
| 虚假签名/双签 | 证书冲突 + 验签成功 | 罚没 + 驱逐验证人 | 重新抵押并通过治理复核 |
| 多号协同操纵投票 | 关联证据达到阈值 | 治理权冻结 + 权重清零周期 | 申诉通过 + 观察期完成 |
| 经济套利攻击 | 异常套利轨迹 | 资产冻结 + 罚没 | 偿付损失 + 再评估 |

### 3.5 申诉与复核
- 任意惩罚必须进入治理审计队列，提供可验证证据哈希。
- 申诉窗口内维持“受限但可申诉”状态，防止先执行后申诉无效。

## 4. 可观测性、SLO 与运行门禁

### 4.1 核心指标
- `deterministic_replay_mismatch_rate`（目标 0）
- `tick_certificate_verify_failure_rate`（目标 0）
- `governance_early_apply_attempts`（应全部被拒绝）
- `sybil_suspect_false_positive_rate`（需持续下降）

### 4.2 发布门禁
- 无证书 tick 不可进入“已确认历史”。
- 任一 `epoch` 边界升级必须有 `Queued` 证据链和签名证书。
- 紧急权限使用必须有完整审计记录，否则发布阻断。

## 5. 验收标准（PRD-GAME-005）

### PRD-GAME-005-01 执行共识层
- AC-005-01: 产出 `TickBlock + TickCertificate`，并能从快照回放校验 `state_root` 一致。
- AC-005-02: 同输入在多次回放中 `events_hash` 与 `state_root` 完全一致。

### PRD-GAME-005-02 治理共识层
- AC-005-03: 所有规则变更均可追溯到治理事件链，禁止旁路修改。
- AC-005-04: 提案只能在 `epoch` 边界生效，提前应用请求必须拒绝。
- AC-005-05: 紧急刹车/否决触发均满足宪章门槛并留存证据。

### PRD-GAME-005-03 身份与反女巫
- AC-005-06: 身份权重由抵押+信誉共同决定，并具备快照冻结。
- AC-005-07: 惩罚流程具备“证据 -> 处置 -> 申诉 -> 复核”完整状态链。

## 5.1 当前实现切片（2026-03-06）
- 已完成 `PRD-GAME-005-01` 的首个实现切片：
  - `World::step/step_with_modules` 在 tick 末尾写入 `TickConsensusRecord`。
  - `snapshot/save/load/from_snapshot(replay)` 持久化并恢复 tick 共识记录。
  - 增加 `verify_tick_consensus_chain()` 对 `parent_hash/events_hash/block_hash/state_root` 的一致性校验。
- 已完成 `PRD-GAME-005-02` 的治理门禁切片：
  - `GovernanceEvent` 增加 `Queued`、`EmergencyBrakeActivated/Released`、`EmergencyVetoed`，治理关键状态变更进入事件流回放。
  - `Proposal` 增加 `queued_at_tick/not_before_tick/activate_epoch/timelock_ticks`，在 `apply_proposal_with_finality` 执行 `timelock + epoch` 门禁校验。
  - 新增治理执行策略 `GovernanceExecutionPolicy`，约束 epoch 长度、激活延迟、紧急权限阈值与最长刹车时长。
  - 增加紧急刹车/释放/否决 API，并对 guardian 阈值、签名身份、时长上限执行拒绝校验。
- 验证口径：
  - `runtime::tests::basic::tick_consensus_records_*`
  - `runtime::tests::governance::{governance_timelock_blocks_early_apply, governance_epoch_gate_blocks_early_apply, governance_emergency_brake_and_release_gate_apply, governance_emergency_veto_rejects_queued_proposal, governance_emergency_controls_reject_invalid_guardian_signatures}`
  - `runtime::tests::persistence::persist_and_restore_world`
  - `runtime::tests::audit::audit_filter_governance_events`
  - `runtime::tests::gameplay_protocol::*` 回归无破坏

## 6. Validation & Decision Record

### 6.1 Test Plan & Traceability
| PRD-ID | 对应任务 | 测试层级 | 验证方法 | 回归影响范围 |
| --- | --- | --- | --- | --- |
| PRD-GAME-005-01 | TASK-GAME-DCG-001/002 | `test_tier_required` + `test_tier_full` | 多次回放一致性、证书验签、快照恢复 | world runtime 一致性与恢复能力 |
| PRD-GAME-005-02 | TASK-GAME-DCG-003/004/005/006 | `test_tier_required` | 治理事件收敛、timelock/epoch 门禁、紧急权限阈值与越权拒绝 | 治理安全与规则稳定性 |
| PRD-GAME-005-03 | TASK-GAME-DCG-007/008 | `test_tier_required` + `test_tier_full` | 女巫攻击模拟、惩罚与申诉闭环 | 治理公平性与经济安全 |

### 6.2 Decision Log
| 决策ID | 选定方案 | 备选方案（否决） | 依据 |
| --- | --- | --- | --- |
| DEC-DCG-001 | Tick 级证书链（block/tick certificate） | 仅保留 journal 事件 | 缺少可验证提交证明，不利于多副本审计。 |
| DEC-DCG-002 | 治理变更按 epoch 生效 | 提案通过即刻生效 | 即刻生效会引入 tick 内规则漂移风险。 |
| DEC-DCG-003 | 权重=抵押+信誉复合模型 | 纯抵押或纯信誉 | 单一维度容易被资本或刷号攻击。 |
