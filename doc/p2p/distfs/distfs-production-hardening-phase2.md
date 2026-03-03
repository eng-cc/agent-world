# Agent World Runtime：DistFS 生产化增强（Phase 2）设计文档

## 目标
- 在 `agent_world_distfs` 增加可验证存储挑战（Storage Challenge）闭环能力，为“存储节点收益分配”提供可审计输入。
- 保持 DistFS 本地 CAS 语义与现有复制路径兼容，不引入破坏性接口变更。
- 输出可被 reward runtime/链上结算层直接消费的挑战证明语义与统计结果。

## 范围

### In Scope
- **DPH2-1：挑战任务模型**
  - 新增挑战请求/挑战任务/挑战回执数据结构。
  - 定义版本号与 proof kind，便于后续协议演进。

- **DPH2-2：挑战下发与应答**
  - `LocalCasStore` 支持基于 `content_hash` 下发挑战：
    - 校验 blob 存在与哈希完整性；
    - 基于 seed 与内容哈希做确定性采样窗口；
    - 生成挑战期望样本哈希。
  - `LocalCasStore` 支持回答挑战并产出回执。

- **DPH2-3：挑战回执验签（验真）与语义投影**
  - 增加回执验证函数（字段一致性、时间窗、样本哈希一致）。
  - 增加回执到 `StorageChallengeProofSemantics` 的投影函数，统一跨 crate 数据语义。

- **DPH2-4：奖励结算可消费统计**
  - 增加按节点聚合的挑战统计（total/pass/fail + 失败原因计数）。
  - 为 reward runtime 的 `storage_valid_checks/storage_total_checks` 提供标准输入源。

- **DPH2-5：回归收口**
  - 完成 `agent_world_distfs` 单测回归。
  - 回写项目文档状态与 devlog。

### Out of Scope
- 分布式 VRF 共识挑战调度网络。
- 链上惩罚执行与经济参数治理。
- 零知识证明/PoRep/PoSt 全协议引入。

## 接口 / 数据

### 挑战请求与任务（草案）
```rust
StorageChallengeRequest {
  challenge_id: String,
  world_id: String,
  node_id: String,
  content_hash: String,
  max_sample_bytes: u32,
  issued_at_unix_ms: i64,
  challenge_ttl_ms: i64,
  vrf_seed: String,
}

StorageChallenge {
  version: u64,
  challenge_id: String,
  world_id: String,
  node_id: String,
  content_hash: String,
  sample_offset: u64,
  sample_size_bytes: u32,
  expected_sample_hash: String,
  issued_at_unix_ms: i64,
  expires_at_unix_ms: i64,
  vrf_seed: String,
}
```

### 挑战回执与校验（草案）
```rust
StorageChallengeReceipt {
  version: u64,
  challenge_id: String,
  node_id: String,
  content_hash: String,
  sample_offset: u64,
  sample_size_bytes: u32,
  sample_hash: String,
  responded_at_unix_ms: i64,
  sample_source: StorageChallengeSampleSource,
  failure_reason: Option<StorageChallengeFailureReason>,
  proof_kind: String,
}

verify_storage_challenge_receipt(challenge, receipt, allowed_clock_skew_ms)
```

### 统计输出（草案）
```rust
NodeStorageChallengeStats {
  node_id: String,
  total_checks: u64,
  passed_checks: u64,
  failed_checks: u64,
  failures_by_reason: BTreeMap<StorageChallengeFailureReason, u64>,
}
```

## 里程碑
- **DPH2-M1**：文档与任务拆解完成。
- **DPH2-M2**：挑战下发/应答落地。
- **DPH2-M3**：回执验真与语义投影落地。
- **DPH2-M4**：统计聚合与测试落地。
- **DPH2-M5**：回归与文档收口。

## 风险
- 挑战采样窗口算法若不稳定会导致跨节点结果不一致；需固定确定性算法与编码。
- 时间窗校验若过严可能误判时钟偏差；通过 `allowed_clock_skew_ms` 参数控制。
- 当前仍是样本哈希挑战，不等价于完整 PoRep/PoSt；需在后续阶段扩展更强证明协议。
