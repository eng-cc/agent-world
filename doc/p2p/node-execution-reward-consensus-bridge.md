# Agent World Runtime：节点执行桥接与奖励共识触发闭环（设计文档）

## 目标
- 把 `agent_world_node` 的共识提交高度与 `agent_world` 运行时执行状态打通，补齐“提交高度 -> 执行状态 -> 存储落盘”的桥接链路。
- 将奖励结算从本地线程直触发升级为“基于网络传播的结算包触发”，使多节点可消费同一结算动作。
- 为奖励采样引入“多观察者 + 签名轨迹（trace）”输入，降低单观察点自报风险，提升可验证性与审计能力。

## 范围

### In Scope
- **ERCB-1：节点提交高度到执行状态桥接**
  - 在 `world_viewer_live` reward runtime 内引入执行桥接器：
    - 监听 `NodeRuntime` 的 `committed_height` 变化；
    - 对每个新提交高度驱动一次 `RuntimeWorld` 执行步进；
    - 产出并落盘执行记录（execution block record），包含 `height/state_root/block_hash/journal_len`；
    - 将快照与日志写入 DistFS CAS（本地存储根）。
  - 增加桥接状态持久化文件，支持重启恢复。

- **ERCB-2：奖励结算网络共识触发**
  - 新增奖励网络消息：结算包 `RewardSettlementEnvelope`。
  - 在配置了 replication libp2p 的情况下：
    - 仅由 sequencer 角色在 epoch 结算点发布结算包；
    - 所有节点（含发布者）订阅并应用结算包到 reward runtime world；
    - 结算包通过既有 `ApplyNodePointsSettlementSigned` 主路径 action 落账。
  - 未配置网络时保持本地单节点 fallback 行为。

- **ERCB-3：多观察者签名采样轨迹**
  - 新增观察轨迹消息 `RewardObservationTrace`：
    - 包含观察 payload、`observer_node_id`、`observer_public_key_hex`、签名；
    - 签名算法使用 ed25519（`rewardobs:v1` 前缀）。
  - reward runtime 每轮发布本地 trace，并消费网络 trace：
    - 验签通过后转为 `NodePointsRuntimeObservation` 喂给 collector；
    - 去重并记录当轮观察者集合；
    - 将 trace 审计信息写入 epoch 报表。

- **ERCB-4：观测与配置**
  - 新增 CLI 参数：`--reward-runtime-min-observer-traces`。
  - 报表新增字段：
    - `execution_bridge_state`
    - `reward_observation_traces`
    - `reward_settlement_transport`

### Out of Scope
- 跨组织 BFT 奖励提案投票与链上治理。
- 跨世界分片的奖励汇总与原子清算。
- 完整 PoRep/PoSt/VRF 协议实现（本期仅提供签名轨迹 + 多观察者输入基础）。

## 接口 / 数据

### 1) 执行桥接状态（新增）
```rust
ExecutionBridgeState {
  last_applied_committed_height: u64,
  last_execution_block_hash: Option<String>,
  last_execution_state_root: Option<String>,
  last_node_block_hash: Option<String>,
}
```

### 2) 执行桥接记录（新增）
```rust
ExecutionBridgeRecord {
  world_id: String,
  height: u64,
  node_block_hash: Option<String>,
  execution_block_hash: String,
  execution_state_root: String,
  journal_len: usize,
  snapshot_ref: String,
  journal_ref: String,
  timestamp_ms: i64,
}
```

### 3) 观察轨迹消息（新增）
```rust
RewardObservationTrace {
  version: u8,
  world_id: String,
  observer_node_id: String,
  observer_public_key_hex: String,
  payload: RewardObservationPayload,
  payload_hash: String,
  signature: String, // rewardobs:v1:<sig_hex>
}
```

### 4) 结算包消息（新增）
```rust
RewardSettlementEnvelope {
  version: u8,
  world_id: String,
  epoch_index: u64,
  signer_node_id: String,
  report: EpochSettlementReport,
  mint_records: Vec<NodeRewardMintRecord>,
  emitted_at_unix_ms: i64,
}
```

### 5) 主题命名（新增）
- `aw.<world_id>.reward.observation`
- `aw.<world_id>.reward.settlement`

### 6) 触发规则
- 执行桥接：`committed_height` 增长时按增量高度逐个桥接。
- 结算发布：collector 到达 epoch 结算点且本节点 `role=sequencer` 时发布。
- 结算应用：收到网络结算包后提交 `ApplyNodePointsSettlementSigned` action 并 `step()`。

## 里程碑
- **ERCB-M0**：设计文档 + 项目管理文档。
- **ERCB-M1**：执行桥接器实现（状态恢复、记录落盘、CAS 引用写入）。
- **ERCB-M2**：奖励结算网络触发与结算包应用实现。
- **ERCB-M3**：多观察者签名轨迹实现与 collector 接线。
- **ERCB-M4**：测试回归（`test_tier_required`）+ 文档/devlog 收口。

## 风险
- 执行桥接是 runtime 外围桥接而非共识内核执行，仍存在“共识 block_hash 与执行 block_hash 不同源”的阶段性差异。
- 在网络抖动场景下，观察轨迹和结算包可能乱序/重复，需要幂等去重兜底。
- 若 `--reward-runtime-min-observer-traces` 配置过高，低活跃网络可能延迟结算。
