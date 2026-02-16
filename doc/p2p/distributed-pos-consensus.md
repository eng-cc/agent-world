# Agent World Runtime：以太坊风格 PoS Head 共识（设计文档）

## 目标
- 在现有 `QuorumConsensus` 基础上新增一套“类以太坊 PoS”的 Head 共识引擎。
- 将“按票数计数”升级为“按 stake 加权”的超级多数判定（默认 2/3）。
- 引入 slot/epoch 与 proposer 选择约束，形成更接近 PoS 的提案与投票语义。
- 提供最小 slashing 规则（double vote / surround vote）用于防止明显违规投票。

## 范围

### In Scope
- 新增 `PosConsensus` 引擎（独立于 `QuorumConsensus`）。
- stake 加权投票判定：
  - `approved_stake >= required_stake` => `Committed`
  - `total_stake - rejected_stake < required_stake` => `Rejected`
- slot proposer 选择（按 stake 加权的确定性选择）。
- 最小 slashing 检查：
  - 同 target_epoch 的不同 block 投票（double vote）
  - source/target 包围关系冲突（surround vote）
- DHT 发布门控：仅在 `Committed` 后写入 world head。
- 快照持久化与恢复（含版本校验）。

### Out of Scope
- 完整以太坊信标链流程（fork choice、execution payload、同步委员会、finalized checkpoint 全量语义）。
- 真实 BLS 签名聚合与链上 slashing 惩罚执行。
- 经济学参数（质押生命周期、退出队列、罚没资金流转）。

## 接口 / 数据

### 核心类型
- `PosValidator { validator_id, stake }`
- `PosConsensusConfig`
  - `validators`
  - `supermajority_numerator`（默认 2）
  - `supermajority_denominator`（默认 3）
  - `epoch_length_slots`（默认 32）
- `PosConsensusStatus`
  - `Pending | Committed | Rejected`
- `PosAttestation`
  - `validator_id`
  - `approve`
  - `source_epoch`
  - `target_epoch`
  - `voted_at_ms`
  - `reason`
- `PosHeadRecord`
  - 记录 head、slot/epoch、attestation 集合与 stake 统计
- `PosConsensusDecision`
  - 对外返回状态与 stake 统计（approved/rejected/required）

### 主流程
- `propose_head(head, proposer_id, slot, proposed_at_ms)`
  - 校验 proposer 是有效 validator。
  - 校验 proposer 等于该 slot 的期望 proposer。
  - 记录提案并自动写入 proposer 的 approve attestation。
- `attest_head(...)`
  - 校验 validator 与提案匹配。
  - 执行 slashing 检查（double/surround）。
  - 写入 attestation 并重算 stake 判定状态。

### DHT 门控
- `propose_world_head_with_pos(dht, ...)`
- `attest_world_head_with_pos(dht, ...)`
- 仅 `Committed` 时执行 `dht.put_world_head(...)`。

## 里程碑
- POS-1：设计文档与项目管理文档落地。
- POS-2：实现 `PosConsensus`、门控方法、快照持久化与单元测试。
- POS-3：回归测试、文档状态收口与 devlog。

## 风险
- 当前 proposer 选择为确定性加权方案，不包含链上随机性来源与抗操纵机制。
- slashing 仅做“检测并拒绝”，不含经济惩罚与全网广播惩戒。
- 与现有 node 主循环尚未接线，需要下一阶段接入执行路径。
