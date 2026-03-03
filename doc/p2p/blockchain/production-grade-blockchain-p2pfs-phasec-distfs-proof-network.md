# Agent World Runtime：生产级区块链 + P2P FS 路线图 Phase C（PRG-M5 跨节点 DistFS 挑战证明网络化）

## 目标
- 将存储可用性证明从“本地 self-challenge/self-answer”升级为“跨节点 challenge-response 网络证明”。
- 在现有 reward runtime 中建立可审计的 DistFS challenge request/proof 通道，使存储节点收益依赖跨节点可验证证明。
- 保持当前系统预算池主导的奖励结算模型，不引入需求侧交易市场。

## 范围

### In Scope（本轮）
- **PRG-C1：挑战请求/证明消息协议化**
  - 新增 DistFS challenge request/proof 主题与消息结构。
  - 增加 request/proof 编解码、签名与验签。

- **PRG-C2：跨节点 challenge-response 运行时**
  - 新增 `DistfsChallengeNetworkDriver`：
    - 维护待确认 challenge 队列；
    - 发布 challenge request；
    - 消费目标节点 proof 响应并验收；
    - 处理超时与失败原因归类。
  - 网络模式下优先走跨节点证明，不再默认本地 self-challenge。

- **PRG-C3：reward runtime 接线与报表**
  - 在 storage 角色下接入网络挑战驱动结果到 `NodePointsRuntimeObservation`。
  - 将 `latest_proof_semantics` 映射到 `storage_challenge_proof_hint`。
  - 报表新增 `distfs_network_challenge` 审计字段。

- **PRG-C4：测试与回归**
  - 新增 challenge/proof 协议单测与双节点闭环单测。
  - 执行 `test_tier_required` 回归并收口文档/devlog。

### Out of Scope
- 需求侧订单撮合与交易化支付市场（PRG-M6 已移除）。
- 完整 PoRep/PoSt 协议（本期为 challenge-response+签名验证闭环）。

## 接口 / 数据

### 1) 主题命名（新增）
- `aw.<world_id>.distfs.challenge.request`
- `aw.<world_id>.distfs.challenge.proof`

### 2) Request Envelope（新增）
```rust
DistfsChallengeRequestEnvelope {
  version: u8,
  world_id: String,
  challenger_node_id: String,
  challenger_public_key_hex: String,
  target_node_id: String,
  challenge: StorageChallenge,
  emitted_at_unix_ms: i64,
  signature: String, // distfschreq:v1:<sig_hex>
}
```

### 3) Proof Envelope（新增）
```rust
DistfsChallengeProofEnvelope {
  version: u8,
  world_id: String,
  responder_node_id: String,
  responder_public_key_hex: String,
  challenge: StorageChallenge,
  receipt: StorageChallengeReceipt,
  emitted_at_unix_ms: i64,
  signature: String, // distfschproof:v1:<sig_hex>
}
```

### 4) 运行时报告（新增）
```rust
DistfsChallengeNetworkTickReport {
  mode: "network" | "fallback_local",
  request_topic: String,
  proof_topic: String,
  known_storage_targets: Vec<String>,
  issued_challenge_ids: Vec<String>,
  answered_challenge_ids: Vec<String>,
  accepted_proof_ids: Vec<String>,
  timed_out_challenge_ids: Vec<String>,
  probe_report: Option<StorageChallengeProbeReport>,
}
```

## 里程碑
- **PRG-CM0**：Phase C 文档与任务拆解。
- **PRG-CM1**：challenge/proof 消息协议与签名验签落地。
- **PRG-CM2**：跨节点 challenge-response driver 落地。
- **PRG-CM3**：reward runtime 接线与报表输出。
- **PRG-CM4**：回归测试与文档/devlog 收口。

## 风险
- 低活跃网络下 proof 回包延迟会导致短时 `total_checks=0` 或 timeout，需要在阈值策略上容忍。
- challenge/proof 双向签名增加排障成本，需要报表输出完整 challenge_id 轨迹。
- 若不同节点分片数据不一致，跨节点 challenge 会出现高失败率，需要结合复制策略与可用数据集治理。
